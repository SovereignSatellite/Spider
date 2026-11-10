open Definitions

type 'x t = 'x type_arena

let define arena definition =
  if arena.count = Array.length arena.definitions then begin
    let grown = Array.make (2 * arena.count) definition in
    Array.blit arena.definitions 0 grown 0 arena.count;
    arena.definitions <- grown
  end;
  arena.definitions.(arena.count) <- definition;
  arena.count <- arena.count + 1;
  Type.numbered (arena.count - 1)

let intern_unlocked arena structural_type =
  match StructuralType.Table.find_opt arena.interned structural_type with
  | Some existing -> existing
  | None ->
      let created = define arena (Structural structural_type) in
      StructuralType.Table.add arena.interned structural_type created;
      created

let create extension =
  let arena =
    {
      definitions = Array.make 16 (TypeDefinition.Box Type.unit);
      count = 0;
      interned = StructuralType.Table.create 64;
      extended = Hashtbl.create 8;
      lock = Mutex.create ();
      extension;
    }
  in
  List.iter
    (fun structural_type -> ignore (intern_unlocked arena structural_type))
    [
      Integer 8;
      Integer 16;
      Integer 32;
      Integer 64;
      Number 32;
      Number 64;
      Sum [ [] ];
      Sum [ []; [] ];
      Array Type.i8;
      Opaque;
    ];
  arena

let intern arena structural_type = Mutex.protect arena.lock (fun () -> intern_unlocked arena structural_type)

let extension arena definition =
  Mutex.protect arena.lock (fun () ->
      match Hashtbl.find_opt arena.extended definition with
      | Some existing -> existing
      | None ->
          let created = define arena (Extension definition) in
          Hashtbl.add arena.extended definition created;
          created)

let create_box arena contents =
  let box_type = define arena (Box Type.unit) in
  arena.definitions.((box_type :> int)) <- Box (contents box_type);
  box_type

let definition arena (described : Type.t) = arena.definitions.((described :> int))

let rec describe arena described =
  match definition arena described with
  | Box _ -> "box" ^ string_of_int (described :> int)
  | Structural structural_type -> describe_structural arena structural_type
  | Extension definition -> arena.extension.describe definition

and describe_structural arena structural_type =
  match structural_type with
  | Integer bit_width -> "i" ^ string_of_int bit_width
  | Number bit_width -> "f" ^ string_of_int bit_width
  | Sum variants ->
      "[" ^ String.concat " | " (List.map (fun fields -> "(" ^ describe_all arena fields ^ ")") variants) ^ "]"
  | Function (parameters, results) ->
      "fn(" ^ describe_all arena parameters ^ ") -> (" ^ describe_all arena results ^ ")"
  | Array element -> "array(" ^ describe arena element ^ ")"
  | Opaque -> "opaque"

and describe_all arena described = String.concat ", " (List.map (describe arena) described)

let element arena array_type =
  match definition arena array_type with
  | Structural (Array element) -> element
  | _ -> invalid_arg ("Expected an array but found " ^ describe arena array_type ^ ".")

let contents arena box_type =
  match definition arena box_type with
  | Box contents -> contents
  | _ -> invalid_arg ("Expected a box but found " ^ describe arena box_type ^ ".")

let signature arena function_type =
  match definition arena function_type with
  | Structural (Function (parameters, results)) -> (parameters, results)
  | _ -> invalid_arg ("Expected a function but found " ^ describe arena function_type ^ ".")

let variants arena sum_type =
  match definition arena sum_type with
  | Structural (Sum variants) -> variants
  | _ -> invalid_arg ("Expected a sum but found " ^ describe arena sum_type ^ ".")
