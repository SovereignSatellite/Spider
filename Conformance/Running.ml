open Plan

type outcome = Returned of int64 list | Trapped | Uninstantiated
type run = Finished of string | Stopped of string * string
type target = { name : string; write : string -> plan -> unit; run : string -> run }

let timeout = 120.

(** The code a shell reports for a process the signal ended. *)
let signalled signal = 128 + Sys.signal_to_int signal

let leading = "--leader"

let lead () =
  match Array.to_list Sys.argv with
  | _ :: flag :: directory :: program :: arguments when flag = leading ->
      ignore (Unix.setsid ());
      Sys.chdir directory;
      Unix.execvp program (Array.of_list (program :: arguments))
  | _ -> ()

let running = Atomic.make []

let rec change update =
  let current = Atomic.get running in
  if not (Atomic.compare_and_set running current (update current)) then change update

let kill process = try Unix.kill (-process) Sys.sigkill with Unix.Unix_error (ESRCH, _, _) -> ()

let interrupted _ =
  List.iter kill (Atomic.get running);
  exit 130

let rec waited process deadline =
  match Unix.waitpid [ WNOHANG ] process with
  | 0, _ when Unix.gettimeofday () > deadline ->
      kill process;
      ignore (Unix.waitpid [] process);
      None
  | 0, _ ->
      Unix.sleepf 0.02;
      waited process deadline
  | _, status -> Some status

let installed program =
  let executable path = try Unix.access path [ X_OK ] = () with Unix.Unix_error _ -> false in
  if String.contains program '/' then executable program
  else
    List.exists
      (fun directory -> executable (Filename.concat directory program))
      (String.split_on_char ':' (Option.value ~default:"" (Sys.getenv_opt "PATH")))

let collected path =
  let text = In_channel.with_open_bin path In_channel.input_all in
  Sys.remove path;
  text

let lines text = List.filter (fun line -> line <> "") (String.split_on_char '\n' text)

let started directory program arguments =
  let output = Filename.temp_file "output" ".txt" in
  let error = Filename.temp_file "error" ".txt" in
  let writing path = Unix.openfile path [ O_WRONLY; O_TRUNC ] 0o600 in
  let output_descriptor = writing output and error_descriptor = writing error in
  let process =
    Unix.create_process Sys.executable_name
      (Array.of_list ([ Sys.executable_name; leading; directory; program ] @ arguments))
      Unix.stdin output_descriptor error_descriptor
  in
  change (List.cons process);
  Unix.close output_descriptor;
  Unix.close error_descriptor;
  let status = waited process (Unix.gettimeofday () +. timeout) in
  change (List.filter (( <> ) process));
  let printed = collected output in
  let complaint = collected error in
  match status with
  | None -> Stopped (printed, program ^ " ran longer than " ^ Printf.sprintf "%.0f" timeout ^ "s")
  | Some (WEXITED 0) -> Finished printed
  | Some status ->
      let code = match status with WEXITED code -> code | WSIGNALED signal | WSTOPPED signal -> signalled signal in
      Stopped
        ( printed,
          program ^ " exited with " ^ string_of_int code ^ ": " ^ String.concat " " (List.take 4 (lines complaint)) )

let run directory program arguments =
  if installed program then started directory program arguments else Stopped ("", program ^ " is not installed")

let returned number results =
  ( int_of_string number,
    Returned
      (List.filter_map
         (fun hexadecimal -> if hexadecimal = "" then None else Some (Int64.of_string ("0x" ^ hexadecimal)))
         (String.split_on_char ' ' results)) )

let outcome line =
  match String.split_on_char '\t' line with
  | [ number; "ok"; results ] -> returned number results
  | [ number; "trap" ] -> (int_of_string number, Trapped)
  | [ number; "uninstantiated" ] -> (int_of_string number, Uninstantiated)
  | _ -> failwith ("A program printed an unexpected line: " ^ line)

let outcomes output = Hashtbl.of_seq (Seq.map outcome (List.to_seq (lines output)))

let distinct items =
  List.rev (List.fold_left (fun seen item -> if List.mem item seen then seen else item :: seen) [] items)

let named prefix shapes =
  let distinct = distinct shapes in
  let names = List.mapi (fun index shape -> (shape, prefix ^ "_" ^ string_of_int index)) distinct in
  (distinct, fun shape -> List.assoc shape names)

let callers steps =
  named "call" (List.filter_map (function Invoke invocation -> Some invocation.signature | _ -> None) steps)

let readers steps = named "read" (List.filter_map (function Get read -> Some read.kind | _ -> None) steps)

let instances steps =
  List.filter_map (function Instantiate instance | Uninstantiable instance -> Some instance | _ -> None) steps

let write_lines path lines =
  Out_channel.with_open_bin path (fun channel ->
      List.iter
        (fun line ->
          Out_channel.output_string channel line;
          Out_channel.output_char channel '\n')
        lines)

let shown (kind : kind) leaf =
  match kind with
  | Number I64T -> "show_i64(" ^ leaf 0 ^ ")"
  | Number F32T -> "show_f32(" ^ leaf 0 ^ ")"
  | Number F64T -> "show_f64(" ^ leaf 0 ^ ")"
  | External -> "show_external(" ^ leaf 0 ^ ", " ^ leaf 1 ^ ")"
  | Number I32T | Reference -> "show_i32(" ^ leaf 0 ^ ")"
