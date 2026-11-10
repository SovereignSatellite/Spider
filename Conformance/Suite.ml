let repository = "https://github.com/RyanLamansky/dotnet-webassembly.git"
let commit = "70c46c0ee78abbdca92490a1881fe509605c5c3d"
let folder = "WebAssembly.Tests/Runtime/SpecTestData"

let rec worktree directory =
  if Sys.file_exists (Filename.concat directory ".git") then directory
  else if Filename.dirname directory = directory then failwith "The conformance harness runs from inside its worktree."
  else worktree (Filename.dirname directory)

let artifacts = Filename.concat (worktree (Filename.dirname (Unix.realpath Sys.executable_name))) "Artifacts"
let clone = Filename.concat artifacts "Conformance"

let git arguments =
  let reading, writing = Unix.pipe ~cloexec:true () in
  let process = Unix.create_process "git" (Array.of_list ("git" :: arguments)) Unix.stdin Unix.stdout writing in
  Unix.close writing;
  let channel = Unix.in_channel_of_descr reading in
  let error = In_channel.input_all channel in
  In_channel.close channel;
  match Unix.waitpid [] process with _, WEXITED 0 -> Ok () | _ -> Error error

let run arguments =
  match git arguments with Ok () -> () | Error error -> failwith ("git " ^ String.concat " " arguments ^ ": " ^ error)

let download () =
  if not (Sys.file_exists clone) then begin
    run [ "clone"; "--quiet"; "--filter=blob:none"; "--sparse"; "--no-checkout"; repository; clone ];
    run [ "-C"; clone; "sparse-checkout"; "set"; "--no-cone"; "/" ^ folder ^ "/" ]
  end;
  if Result.is_error (git [ "-C"; clone; "cat-file"; "-e"; commit ^ "^{commit}" ]) then
    run [ "-C"; clone; "fetch"; "--quiet"; "--filter=blob:none"; "origin"; commit ];
  run [ "-C"; clone; "checkout"; "--quiet"; commit ];
  Filename.concat clone folder

type value =
  | Bits of Wasm.Types.numtype * int64
  | CanonicalNaN of Wasm.Types.numtype
  | ArithmeticNaN of Wasm.Types.numtype
  | Reference of bool
  | External of int64
  | Unsupported of string

module Value = struct
  let describe value =
    match value with
    | Bits (_, bits) -> Printf.sprintf "%Lx" bits
    | CanonicalNaN _ -> "nan:canonical"
    | ArithmeticNaN _ -> "nan:arithmetic"
    | Reference true -> "ref.null"
    | Reference false -> "non-null"
    | External number -> "ref.extern " ^ Int64.to_string number
    | Unsupported value_type -> value_type
end

type action_kind = Invoke | Get
type action = { kind : action_kind; module_ : string option; field : string; arguments : value list }

type command =
  | Module of { binary : string option; name : string option }
  | ModuleInstance of { definition : string; instance : string option }
  | ModuleDefinition of { binary : string; name : string option }
  | Register of { name : string option; registered : string }
  | AssertUninstantiable of string
  | AssertReturn of action * value list
  | AssertTrap of action * string
  | Action of action
  | Unplanned of string

type test = { name : string; commands : (int * command) list }

let value element =
  let parse value_type width =
    match Json.text (Json.get element "value") with
    | "nan:canonical" -> CanonicalNaN value_type
    | "nan:arithmetic" -> ArithmeticNaN value_type
    | number when number.[0] = '-' ->
        Bits (value_type, Int64.logand (Int64.of_string number) (Int64.shift_right_logical (-1L) (64 - width)))
    | number -> Bits (value_type, Int64.of_string ("0u" ^ number))
  in
  match Json.text (Json.get element "type") with
  | "i32" -> parse I32T 32
  | "i64" -> parse I64T 64
  | "f32" -> parse F32T 32
  | "f64" -> parse F64T 64
  | "funcref" -> Reference (Option.is_some (Json.property element "value"))
  | "refnull" | "nullfuncref" -> Reference true
  | "externref" -> (
      match Json.property element "value" with
      | None -> Reference false
      | Some number when Json.text number = "null" -> Reference true
      | Some number -> External (Int64.of_string (Json.text number)))
  | other -> Unsupported other

let values command name = match Json.property command name with Some (List list) -> List.map value list | _ -> []
let text element name = Option.map Json.text (Json.property element name)

let action element =
  let action = Json.get element "action" in
  {
    kind =
      (match Json.text (Json.get action "type") with
      | "invoke" -> Invoke
      | "get" -> Get
      | other -> failwith ("An unexpected action: " ^ other));
    module_ = text action "module";
    field = Json.text (Json.get action "field");
    arguments = values action "args";
  }

let command folder element =
  let binary =
    match (text element "filename", text element "module_type") with
    | Some name, Some "binary" -> Some (Filename.concat folder name)
    | _ -> None
  in
  let name = match text element "name" with None -> text element "module" | name -> name in
  ( (match Json.get element "line" with Number number -> number | _ -> invalid_arg "Expected a line."),
    match Json.text (Json.get element "type") with
    | "module" -> Module { binary; name }
    | "module_instance" -> ModuleInstance { definition = Option.get name; instance = text element "instance" }
    | "module_definition" -> ModuleDefinition { binary = Option.get binary; name }
    | "register" -> Register { name; registered = Json.text (Json.get element "as") }
    | "assert_uninstantiable" -> AssertUninstantiable (Option.get binary)
    | "assert_return" -> AssertReturn (action element, values element "expected")
    | "assert_trap" -> AssertTrap (action element, Json.text (Json.get element "text"))
    | "action" -> Action (action element)
    | other -> Unplanned other )

let rec files directory =
  List.concat_map
    (fun entry ->
      let path = Filename.concat directory entry in
      if Sys.is_directory path then files path else if Filename.check_suffix entry ".json" then [ path ] else [])
    (Array.to_list (Sys.readdir directory))

let tests root =
  List.map
    (fun file ->
      let document = Json.parse (In_channel.with_open_bin file In_channel.input_all) in
      let relative = String.sub file (String.length root + 1) (String.length file - String.length root - 1) in
      {
        name = Filename.remove_extension relative;
        commands =
          (match Json.get document "commands" with
          | List commands -> List.map (command (Filename.dirname file)) commands
          | _ -> []);
      })
    (List.sort compare (files root))
