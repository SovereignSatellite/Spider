open IR
open Definitions
open Allocator
open Emission
open Tree
open LuaJITExtension

let discard = Name.Text "_"
let stack = Name.Text "stack"
let reserved_locals = List.length [ discard; stack ]

(* LuaJIT: a function has 250 slots, LuaJIT is untyped, and registers group by Lua type. *)
let luajit_registers = 250

type frame = { context : Context.t; allocation : x Allocation.t; reader : Operations.reader }

let path (register : Register.t) : Expression.t =
  match register with
  | Fixed _ -> Name (Register register)
  | Overflow slot -> Index (Name stack, Integer (Int64.of_int (slot + 1)))

(* LuaJIT: a function has 250 slots, LuaJIT is untyped, and registers group by Lua type. *)
let read register _ = path register

let create_frame (context : Context.t) allocation =
  let reader =
    Operations.Conditions.reader context.layouts allocation Operations.select (Operations.test context)
      (Operations.compute context) read (fun _ -> Expression.Nil)
  in
  { context; allocation; reader }

module Traversal = Traversal.Make (struct
  type nonrec x = x
  type nonrec frame = frame
  type expression = Expression.t
  type statement = Statement.t

  let selects_once = false
  let layouts frame = frame.context.layouts
  let allocation frame = frame.allocation
  let reader frame = frame.reader

  (* LuaJIT: a function has 250 slots, LuaJIT is untyped, and registers group by Lua type. *)
  let declare _ (register : Register.t) : statement list =
    match register with Fixed _ -> [ Local (Register register, None) ] | Overflow _ -> []

  let compute frame = Operations.compute frame.context frame.reader

  (* LuaJIT: a function has 250 slots, LuaJIT is untyped, and registers group by Lua type. *)
  let read _ = read
  let path = path
  let unused : expression = Name discard

  let assign (targets, values) : statement =
    let rec trimmed reversed =
      match reversed with target :: rest when target = unused -> trimmed rest | _ -> reversed
    in
    let kept = List.rev (trimmed (List.rev targets)) in
    let assignment = Statement.Assign (kept, values) in
    (* LuaJIT: a function has 250 slots, LuaJIT is untyped, and registers group by Lua type. *)
    if List.mem unused kept then Do [ Local (discard, None); assignment ] else assignment

  let discard (value : expression) : statement =
    match value with
    | Call _ -> Call value
    (* LuaJIT: LuaJIT is untyped. *)
    | _ -> Do [ Local (discard, Some value) ]

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
      | Some register -> Name.Register register
      | None -> discard)
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
  | None -> invalid_arg ("The LuaJIT target overflows at most 16384 slots, not " ^ string_of_int count ^ ".")

let acquire_stack_around frame body count =
  let size = size_class_of count in
  let acquired = Operations.call_helper frame.context "StackAcquire" [ size ] in
  let released = Operations.call_helper frame.context "StackRelease" [ Name stack; size ] in
  (* LuaJIT: a function has 250 slots, LuaJIT is untyped, and registers group by Lua type. *)
  (Statement.Local (stack, Some acquired) :: body) @ [ Statement.Call released ]

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

(* LuaJIT: a function has 250 slots, LuaJIT is untyped, and registers group by Lua type. *)
let tag types leaf =
  match TypeArena.definition types leaf with
  | Structural (Integer 64) | Extension IR_LuaJIT.Unsigned -> "cdata"
  | Structural (Integer _ | Number _) -> "number"
  | Structural (Function _) -> "function"
  | Structural (Array element) when element = Type.i8 -> "cdata"
  | Structural Opaque -> "unknown"
  | Structural (Array _ | Sum _) | Box _ | Extension _ -> "table"

let rec widest_call (context : Context.t) region =
  Array.fold_left
    (fun widest node ->
      let leaves value_types = List.length (Layouts.leaves context.layouts value_types) in
      let own =
        match node.kind with
        | FunctionApply | Extension `LuaJITTry ->
            1
            + leaves (List.map (Link.type_ (Layouts.types context.layouts)) (Array.to_list node.inputs))
            + leaves (Node.output_types (Layouts.types context.layouts) node)
        | _ -> 0
      in
      List.fold_left (fun widest inner -> max widest (widest_call context inner)) (max widest own) (Node.regions node))
    0 (Passes.RegionNodes.ordered region)

let available_locals context maximum_locals definition =
  max 1 (min maximum_locals (luajit_registers - widest_call context definition) - reserved_locals)

let build (context : Context.t) maximum_locals definition : Expression.t =
  let allocation =
    Allocation.allocate context.layouts
      (* LuaJIT: a function has 250 slots, LuaJIT is untyped, and registers group by Lua type. *)
      (RegisterFile.create (available_locals context maximum_locals definition) (tag (Layouts.types context.layouts)))
      (Traversal.reads_once context.layouts) Operations.computed_where_read definition
  in
  let frame = create_frame context allocation in
  let parameters = parameters frame definition in
  let label = Computation.label context.labels (fun text -> Statement.Comment text) definition in
  let body = with_stack frame (spilled_parameters frame definition @ Traversal.body frame definition) in
  Function (parameters, Declarations.placed (label @ body @ return_results frame definition))
