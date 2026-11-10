open Suite
open Plan
open Running

let unoptimized = Passes.Options.resolve 0 []

let checked (omission : Passes.Options.optimization) =
  Passes.Options.resolve Passes.Options.highest [ (Optimization omission.name, false) ]

let targets =
  [
    RustHarness.target "rust" (checked Rust.Storage.omission);
    RustHarness.target "rust-unoptimized" unoptimized;
    LuauHarness.target "luau" (checked Luau.Storage.omission);
    LuauHarness.target "luau-unoptimized" unoptimized;
    LuaJITHarness.target "luajit" (checked LuaJIT.Storage.omission);
    LuaJITHarness.target "luajit-unoptimized" unoptimized;
  ]

let matches expected bits =
  let quiet (value_type : Wasm.Types.numtype) =
    match value_type with I32T | F32T -> 0x7FC0_0000L | I64T | F64T -> 0x7FF8_0000_0000_0000L
  in
  let magnitude (value_type : Wasm.Types.numtype) =
    match value_type with I32T | F32T -> 0x7FFF_FFFFL | I64T | F64T -> 0x7FFF_FFFF_FFFF_FFFFL
  in
  match expected with
  | Bits (_, expected_bits) -> Int64.equal bits expected_bits
  | CanonicalNaN value_type -> Int64.equal (Int64.logand bits (magnitude value_type)) (quiet value_type)
  | ArithmeticNaN value_type -> Int64.equal (Int64.logand bits (quiet value_type)) (quiet value_type)
  | Reference is_null -> Int64.equal bits 0L = is_null
  | External number -> Int64.equal bits (Plan.host number)
  | Unsupported _ -> false

let unexpected_return expectation bits =
  let shown = String.concat " " (List.map (Printf.sprintf "%Lx") bits) in
  Some ("returned [" ^ shown ^ "], expected " ^ Expectation.describe expectation)

let judge expectation outcome =
  match (expectation, outcome) with
  | Returns expected, Returned bits when List.length bits = List.length expected && List.for_all2 matches expected bits
    ->
      None
  | Traps, Trapped | Finishes, Returned _ -> None
  | expectation, Returned bits -> unexpected_return expectation bits
  | expectation, Trapped -> Some ("trapped, expected " ^ Expectation.describe expectation)
  | _, Uninstantiated -> Some "its module did not instantiate"

let failures plan run =
  let reported, stopped =
    match run with
    | Finished output -> (outcomes output, "the program finished without reporting it")
    | Stopped (output, reason) -> (outcomes output, reason)
  in
  List.filter_map
    (fun (line, expectation) ->
      let why =
        match Hashtbl.find_opt reported line with Some outcome -> judge expectation outcome | None -> Some stopped
      in
      Option.map (fun why -> plan.test ^ ":" ^ string_of_int line ^ ": " ^ why) why)
    (judged plan)

let folder target = Filename.concat (Filename.concat (Filename.concat artifacts "Staging") "Conformance") target.name

let rec create_directory directory =
  if not (Sys.file_exists directory) then begin
    create_directory (Filename.dirname directory);
    try Sys.mkdir directory 0o755 with Sys_error _ when Sys.file_exists directory -> ()
  end

let rec delete path =
  if (Unix.lstat path).st_kind = S_DIR then begin
    Array.iter (fun entry -> delete (Filename.concat path entry)) (Sys.readdir path);
    Sys.rmdir path
  end
  else Sys.remove path

let write target plan =
  let directory = Filename.concat (folder target) plan.test in
  create_directory directory;
  try
    target.write directory plan;
    Ok directory
  with error -> Error ("writing failed: " ^ Printexc.to_string error)

let execute jobs =
  let runs = Array.make (Array.length jobs) (Finished "") in
  let taken = Atomic.make 0 in
  let rec work () =
    let index = Atomic.fetch_and_add taken 1 in
    if index < Array.length jobs then begin
      let target, plan = jobs.(index) in
      runs.(index) <-
        (match write target plan with Ok directory -> target.run directory | Error reason -> Stopped ("", reason));
      work ()
    end
  in
  Array.iter Domain.join
    (Array.init (min (Domain.recommended_domain_count ()) (Array.length jobs)) (fun _ -> Domain.spawn work));
  runs

let report target plans runs =
  let failures = Array.map2 failures plans runs in
  let failed = List.concat (Array.to_list failures) in
  let passed = Array.fold_left (fun sum plan -> sum + List.length (judged plan)) 0 plans - List.length failed in
  write_lines (Filename.concat (folder target) "failures.txt") failed;
  Array.iter2
    (fun plan failures -> if failures = [] then delete (Filename.concat (folder target) plan.test))
    plans failures;
  List.iter (fun failure -> print_endline (target.name ^ " " ^ failure)) (List.take 20 failed);
  (Printf.sprintf "%s %d passed %d failed" target.name passed (List.length failed), List.length failed)

let count plans pick =
  let counts = Dynarray.create () in
  List.iter
    (fun plan ->
      List.iter
        (fun (_, disposition) ->
          Option.iter
            (fun key ->
              match Dynarray.find_index (fun (seen, _) -> seen = key) counts with
              | Some index ->
                  let _, number = Dynarray.get counts index in
                  Dynarray.set counts index (key, number + 1)
              | None -> Dynarray.add_last counts (key, 1))
            (pick disposition))
        plan.commands)
    plans;
  List.stable_sort (fun (_, left) (_, right) -> compare right left) (Dynarray.to_list counts)

let () =
  Running.lead ();
  Sys.set_signal Sys.sigint (Signal_handle Running.interrupted);
  let names = List.tl (Array.to_list Sys.argv) in
  let started = Unix.gettimeofday () in
  let plans = List.map Plan.create (tests (download ())) in
  let runnable = Array.of_list (List.filter (fun plan -> judged plan <> []) plans) in
  let failed_early =
    List.concat_map
      (fun plan ->
        List.filter_map
          (fun (line, disposition) ->
            match disposition with
            | Failed reason -> Some (plan.test ^ ":" ^ string_of_int line ^ ": " ^ reason)
            | _ -> None)
          plan.commands)
      plans
  in
  List.iter (fun failure -> print_endline ("lifting " ^ failure)) (List.take 20 failed_early);
  let chosen = List.filter (fun target -> names = [] || List.mem target.name names) targets in
  List.iter (fun target -> if Sys.file_exists (folder target) then delete (folder target)) chosen;
  let runs =
    execute
      (Array.of_list
         (List.concat_map (fun target -> List.map (fun plan -> (target, plan)) (Array.to_list runnable)) chosen))
  in
  let count_runnable = Array.length runnable in
  let summaries =
    List.mapi
      (fun index target -> report target runnable (Array.sub runs (index * count_runnable) count_runnable))
      chosen
  in
  let refused = count plans (function Refused feature -> Some feature | _ -> None) in
  let skipped = count plans (function Skipped reason -> Some reason | _ -> None) in
  List.iter (fun (feature, modules) -> Printf.printf "refused %s: %d modules\n" feature modules) refused;
  List.iter (fun (reason, commands) -> Printf.printf "skipped %s: %d commands\n" reason commands) skipped;
  Printf.printf "%s; %d failed to lift; %.0fs\n"
    (String.concat ", " (List.map fst summaries))
    (List.length failed_early)
    (Unix.gettimeofday () -. started);
  exit (if failed_early = [] && List.for_all (fun (_, failed) -> failed = 0) summaries then 0 else 1)
