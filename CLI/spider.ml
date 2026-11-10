open IR
open Cmdliner

let fail message =
  prerr_endline message;
  1

let syntax_error (error : TuringMachine.Parser.syntax_error) =
  match error with
  | UnmatchedOpen position -> "UnmatchedOpen " ^ string_of_int position
  | UnmatchedClose position -> "UnmatchedClose " ^ string_of_int position

let lift types source input =
  let program = In_channel.with_open_bin input In_channel.input_all in
  match source with
  | `WebAssembly -> (
      try Ok (WebAssembly.Module.lift_labelled types program) with
      | WebAssembly.Types.Unsupported feature -> Error (input ^ ": unsupported: " ^ feature)
      | WebAssembly.Types.Invalid reason -> Error (input ^ ": invalid: " ^ reason))
  | `TuringMachine -> (
      match TuringMachine.Parser.parse program with
      | Ok instructions -> Ok (TuringMachine.Lifter.lift types instructions, RegionTable.create 0)
      | Error error -> Error (input ^ ": " ^ syntax_error error))

let build options source input types print =
  match lift types source input with
  | Error message -> fail message
  | Ok (compilation_unit, labels) -> (
      match WellFormedness.check compilation_unit with
      | [] ->
          Passes.Optimizer.run options compilation_unit;
          print_string (print labels compilation_unit);
          0
      | errors -> fail (String.concat "\n" errors))

let compile options source input target =
  match target with
  | `Rust ->
      build options source input (TypeArena.create Rust.RustExtension.extension) (fun labels compilation_unit ->
          Rust.Printer.print (Rust.Build.labelled options labels compilation_unit))
  | `Luau ->
      build options source input (TypeArena.create Luau.LuauExtension.extension) (fun labels compilation_unit ->
          Luau.Printer.print (Luau.Build.labelled options labels compilation_unit))
  | `LuaJIT ->
      build options source input (TypeArena.create LuaJIT.LuaJITExtension.extension) (fun labels compilation_unit ->
          LuaJIT.Printer.print (LuaJIT.Build.labelled options labels compilation_unit))
  | `Json ->
      build options source input (TypeArena.create IR_Memory.extension) (fun _ compilation_unit ->
          Json.Printer.print compilation_unit)

let runtime target =
  print_string
    (match target with
    | `Rust -> Rust.Printer.print (Rust.Build.whole_runtime ())
    | `Luau -> Luau.Printer.print (Luau.Build.whole_runtime ())
    | `LuaJIT -> LuaJIT.Printer.print (LuaJIT.Build.whole_runtime ()));
  0

let optimizations =
  Passes.Catalogue.optimizations @ Rust.Lowering.optimizations @ Luau.Lowering.optimizations
  @ LuaJIT.Lowering.optimizations

let domains_section = "OPTIMIZATION DOMAINS"
let intents_section = "OPTIMIZATION INTENTS"
let domain_section domain = String.uppercase_ascii (Passes.Options.domain_name domain) ^ " OPTIMIZATIONS"

let switch (selector : Passes.Options.selector) name section doc =
  let absent =
    match selector with Domain _ -> "the level" | Intent _ -> "its domain" | Optimization _ -> "its intent"
  in
  Term.(
    const (List.map (fun value -> (selector, value)))
    $ Arg.(value & opt_all bool [] & info [ name ] ~docv:"BOOL" ~docs:section ~absent ~doc))

let intents =
  List.sort_uniq compare
    (List.map
       (fun (optimization : Passes.Options.optimization) -> (optimization.domain, optimization.intent))
       optimizations)

let switches =
  let open Passes.Options in
  List.map
    (fun domain ->
      let name = domain_name domain in
      switch (Domain domain) (name ^ "-optimizations") domains_section
        ("Enable or disable every " ^ name ^ " optimization."))
    domains
  @ List.map
      (fun (domain, intent) ->
        let name = domain_name domain ^ "-" ^ intent_name intent in
        switch
          (Intent (domain, intent))
          name intents_section
          ("Enable or disable " ^ domain_name domain ^ " " ^ intent_name intent ^ "."))
      intents
  @ List.map
      (fun optimization ->
        switch (Optimization optimization.name) optimization.name (domain_section optimization.domain)
          optimization.summary)
      optimizations

let level =
  let highest = Passes.Options.highest in
  Arg.(
    required
    & opt (some (enum (List.init (highest + 1) (fun level -> (string_of_int level, level))))) None
    & info [ "O"; "optimization-level" ] ~docv:"LEVEL"
        ~doc:("Enable the optimizations at or below $(docv), from 0 for none to " ^ string_of_int highest ^ " for all."))

let options =
  Term.(
    const Passes.Options.resolve $ level
    $ List.fold_left (fun settings switch -> const ( @ ) $ settings $ switch) (const []) switches)

let input = Arg.(required & pos 0 (some file) None & info [] ~docv:"INPUT" ~doc:"The program to compile.")

let source =
  Arg.(
    required
    & opt (some (enum [ ("web-assembly", `WebAssembly); ("turing-machine", `TuringMachine) ])) None
    & info [ "s"; "source" ] ~docv:"SOURCE" ~doc:"The language of $(i,INPUT), $(b,web-assembly) or $(b,turing-machine).")

let target targets =
  Arg.(
    required
    & opt (some (enum targets)) None
    & info [ "t"; "target" ] ~docv:"TARGET" ~doc:("The target, " ^ Arg.doc_alts_enum targets ^ "."))

let runtimes = [ ("rust", `Rust); ("luau", `Luau); ("luajit", `LuaJIT) ]

let compile_command =
  let man =
    [
      `S Manpage.s_description;
      `P
        "Compiles $(i,INPUT) and writes the program to standard output. Each optimization belongs to an intent within \
         a domain. Setting an optimization overrides its intent, which overrides its domain, which overrides the \
         level, whatever their order; of settings of one, the last wins.";
      `S domains_section;
      `S intents_section;
    ]
    @ List.map (fun domain -> `S (domain_section domain)) Passes.Options.domains
  in
  Cmd.v
    (Cmd.info "compile" ~doc:"Compile a program." ~man)
    Term.(const compile $ options $ source $ input $ target (("json", `Json) :: runtimes))

let runtime_command =
  Cmd.v
    (Cmd.info "runtime" ~doc:"Write a target's whole runtime, of which each program holds only what it needs.")
    Term.(const runtime $ target runtimes)

let () =
  exit
    (Cmd.eval'
       (Cmd.group
          (Cmd.info "spider" ~doc:"Compile programs to Rust, Luau, LuaJIT or JSON.")
          [ compile_command; runtime_command ]))
