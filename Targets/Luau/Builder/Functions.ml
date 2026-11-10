open IR
open Definitions
open Allocator
open Emission
open Tree
open LuauExtension

let discard = Name.Text "_"
let stack = Name.Text "stack"
let reserved_locals = List.length [ discard; stack ]

(* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
let luau_registers = 255
let declare_discard value = Statement.Local (discard, Named (Text "any"), value)

type frame = {
  context : Context.t;
  allocation : x Allocation.t;
  (* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
  spill_type : LuauType.t;
  types : (Register.t, LuauType.t Lazy.t) Hashtbl.t;
  reader : Operations.reader;
}

(* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
let type_of frame register = Lazy.force (Hashtbl.find frame.types register)

let path (register : Register.t) : Expression.t =
  match register with
  | Fixed _ -> Name (Register register)
  | Overflow slot -> Index (Name stack, Integer (Int64.of_int (slot + 1)))

(* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
let read (context : Context.t) (types : (Register.t, LuauType.t Lazy.t) Hashtbl.t) register leaf_type : Expression.t =
  match Lazy.force (Hashtbl.find types register) with
  | Union _ -> Cast (path register, LuauTypes.leaf_type context.luau leaf_type)
  | _ -> path register

let create_frame (context : Context.t) allocation =
  (* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
  let types_of register = List.map (LuauTypes.leaf_type context.luau) (Allocation.leaf_types allocation register) in
  let spill_type =
    LuauType.union
      (List.concat_map
         (fun (register : Register.t) -> match register with Overflow _ -> types_of register | Fixed _ -> [])
         (Allocation.registers allocation))
  in
  let types = Hashtbl.create 64 in
  (* Lazy, so that each type's boxes are numbered where it is first written. *)
  List.iter
    (fun (register : Register.t) ->
      Hashtbl.replace types register
        (lazy (match register with Fixed _ -> LuauType.union (types_of register) | Overflow _ -> spill_type)))
    (Allocation.registers allocation);
  let reader =
    Operations.Conditions.reader context.layouts allocation Operations.select (Operations.test context)
      (* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
      (Operations.compute context) (read context types) (fun _ -> Expression.Nil)
  in
  { context; allocation; spill_type; types; reader }

module Traversal = Traversal.Make (struct
  type nonrec x = x
  type nonrec frame = frame
  type expression = Expression.t
  type statement = Statement.t

  let selects_once = false
  let layouts frame = frame.context.layouts
  let allocation frame = frame.allocation
  let reader frame = frame.reader

  (* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
  let declare frame (register : Register.t) : statement list =
    match register with Fixed _ -> [ Local (Register register, type_of frame register, None) ] | Overflow _ -> []

  let compute frame = Operations.compute frame.context frame.reader

  (* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
  let read frame = read frame.context frame.types
  let path = path
  let unused : expression = Name discard

  let assign (targets, values) : statement =
    let rec trimmed reversed =
      match reversed with target :: rest when target = unused -> trimmed rest | _ -> reversed
    in
    let kept = List.rev (trimmed (List.rev targets)) in
    let assignment = Statement.Assign (kept, values) in
    (* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
    if List.mem unused kept then Do [ declare_discard None; assignment ] else assignment

  let discard (value : expression) : statement =
    match value with
    | Call _ -> Call value
    (* Luau: Luau is typed. *)
    | _ -> Do [ declare_discard (Some value) ]

  let if_ (condition, body, otherwise) : statement = If (condition, body, otherwise)
  let break_ : statement = Break

  let select tag runs =
    Operations.halved tag
      (fun test below above ->
        match below with [] -> [ Statement.If (test 1, above, []) ] | _ -> [ Statement.If (test 0, below, above) ])
      runs

  let loop body : statement = Loop body
end)

let parameters frame definition =
  List.map
    (fun (link, leaf) ->
      match Allocation.register frame.allocation link leaf with
      (* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
      | Some register -> (Name.Register register, type_of frame register)
      | None ->
          (discard, LuauTypes.leaf_type frame.context.luau (Computation.leaf_type_of frame.context.layouts link leaf)))
    (Computation.output_leaves frame.context.layouts definition.arguments)

let spilled_parameters frame definition =
  List.filter_map
    (fun (link, leaf) ->
      match Allocation.register frame.allocation link leaf with
      | Some (Overflow _ as register) -> Some (Statement.Assign ([ path register ], [ Name (Register register) ]))
      | Some (Fixed _) | None -> None)
    (Computation.output_leaves frame.context.layouts definition.arguments)

let size_class_of count : Expression.t =
  match List.find_opt (fun size -> size >= count) [ 16; 64; 256; 1024; 4096; 16384 ] with
  | Some size -> Integer (Int64.of_int size)
  | None -> invalid_arg ("The Luau target overflows at most 16384 slots, not " ^ string_of_int count ^ ".")

let acquire_stack_around frame body count =
  let size = size_class_of count in
  (* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
  let table_type = LuauType.Sequence frame.spill_type in
  let acquired = Operations.call_helper frame.context "StackAcquire" [ size ] in
  let released = Operations.call_helper frame.context "StackRelease" [ Name stack; size ] in
  (Statement.Const (stack, Some table_type, Cast (acquired, table_type)) :: body) @ [ Statement.Call released ]

let with_stack frame body =
  match
    List.length
      (List.filter
         (fun (register : Register.t) -> match register with Overflow _ -> true | Fixed _ -> false)
         (Allocation.registers frame.allocation))
  with
  | 0 -> body
  | count -> acquire_stack_around frame body count

let return_results frame definition : Statement.t list =
  match Computation.input_values frame.context.layouts frame.reader definition.results 0 with
  | [] -> []
  | returned -> [ Return returned ]

(* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
let rec loops allocation region =
  List.exists
    (fun node ->
      match node.kind with RegionRepeat _ -> true | _ -> List.exists (loops allocation) (Node.regions node))
    (Allocation.evaluated allocation region)

let rec widest_call (context : Context.t) region =
  Array.fold_left
    (fun widest node ->
      let leaves value_types = List.length (Layouts.leaves context.layouts value_types) in
      let own =
        match node.kind with
        | FunctionApply | Extension `LuauTry ->
            1
            + leaves (List.map (Link.type_ (Layouts.types context.layouts)) (Array.to_list node.inputs))
            + leaves (Node.output_types (Layouts.types context.layouts) node)
        | _ -> 0
      in
      List.fold_left (fun widest inner -> max widest (widest_call context inner)) (max widest own) (Node.regions node))
    0 (Passes.RegionNodes.ordered region)

let available_locals context maximum_locals definition =
  max 1 (min maximum_locals (luau_registers - widest_call context definition) - reserved_locals)

let build (context : Context.t) maximum_locals definition : Expression.t =
  let allocation =
    Allocation.allocate context.layouts
      (* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
      (RegisterFile.create (available_locals context maximum_locals definition) (LuauTypes.tag context.luau))
      (Traversal.reads_once context.layouts) Operations.computed_where_read definition
  in
  let frame = create_frame context allocation in
  (* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
  let native = loops allocation definition in
  let parameters = parameters frame definition in
  let results =
    List.map (LuauTypes.leaf_type context.luau) (Layouts.leaves context.layouts (Region.result_types definition))
  in
  let label = Computation.label context.labels (fun text -> Statement.Comment text) definition in
  let body = with_stack frame (spilled_parameters frame definition @ Traversal.body frame definition) in
  (* Luau: a function has 255 registers, Luau is typed, and loops run as native code. *)
  Function (native, parameters, results, Declarations.placed (label @ body @ return_results frame definition))
