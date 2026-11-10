open IR
open Definitions
open Allocator
open Emission
open Tree
open RustExtension

let discard = Name.Text "_"

type frame = { context : Context.t; allocation : x Allocation.t; reader : Operations.reader }

let leaf_type_of frame register =
  match Allocation.leaf_types frame.allocation register with
  | [ leaf_type ] -> leaf_type
  | _ -> invalid_arg "A Rust register holds one leaf_type."

let type_of frame register = RustTypes.leaf_type frame.context.rust (leaf_type_of frame register)
let path register : Expression.t = Name (Register register)
let read register _ = path register

let create_frame (context : Context.t) allocation =
  let reader =
    Operations.Conditions.reader context.layouts allocation Operations.select (Operations.test context)
      (Operations.compute context) read (RustTypes.zero context.rust)
  in
  { context; allocation; reader }

module Traversal = Traversal.Make (struct
  type nonrec x = x
  type nonrec frame = frame
  type expression = Expression.t
  type statement = Statement.t

  let selects_once = true
  let layouts frame = frame.context.layouts
  let allocation frame = frame.allocation
  let reader frame = frame.reader

  let declare frame register : statement list =
    let leaf_type = leaf_type_of frame register in
    let zero = RustTypes.zero frame.context.rust leaf_type in
    [ Local (Register register, RustTypes.leaf_type frame.context.rust leaf_type, zero) ]

  let compute frame = Operations.compute frame.context frame.reader
  let read _ = read
  let path = path
  let unused : expression = Name discard
  let assign (targets, values) : statement = Assign (targets, values)

  let discard (value : expression) : statement =
    match value with Call _ -> Call value | _ -> Assign ([ Name discard ], [ value ])

  let if_ (condition, body, otherwise) : statement = If (condition, body, otherwise)
  let break_ : statement = Break
  let select tag runs = [ Statement.Match (tag, runs) ]
  let loop body : statement = Loop body
end)

let parameters frame definition =
  List.map
    (fun (link, leaf) ->
      match Allocation.register frame.allocation link leaf with
      | Some register -> (Name.Register register, type_of frame register)
      | None ->
          (discard, RustTypes.leaf_type frame.context.rust (Computation.leaf_type_of frame.context.layouts link leaf)))
    (Computation.output_leaves frame.context.layouts definition.arguments)

let return_results frame definition : Statement.t list =
  match Computation.input_values frame.context.layouts frame.reader definition.results 0 with
  | [] -> []
  | returned -> [ Return returned ]

let build (context : Context.t) name definition : Item.t =
  let allocation =
    Allocation.allocate context.layouts (RegisterFile.create 0 Fun.id) (Traversal.reads_once context.layouts)
      Operations.computed_where_read definition
  in
  let frame = create_frame context allocation in
  let parameters = parameters frame definition in
  let result = RustTypes.result_type context.rust (Region.result_types definition) in
  let label = Computation.label context.labels (fun text -> Statement.Comment text) definition in
  let body = Traversal.body frame definition in
  Function (name, parameters, result, label @ body @ return_results frame definition)
