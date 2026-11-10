open IR
open Definitions
open Types

module Field = struct
  type t = Function of int | Memory of int | Data of int | Table of int | Element of int | Global of int | Tag of int

  let compare = compare
end

module Fields = Map.Make (Field)

module Access = struct
  type t = Reads | Writes | Replaces
end

module Callee = struct
  type t = Function of int | Indirect of signature | Host
end

type 'x context = {
  types : 'x TypeArena.t;
  signatures : signatures;
  erased : Type.t;
  imported_functions : int;
  bodies : 'x region array;
  tag_box : Type.t;
  exception_box : Type.t;
  boxes : Type.t -> Type.t;
  effects : (Callee.t, Access.t Fields.t) Hashtbl.t;
}

let result node = Link.output node 0
let make kind inputs = result (Node.create kind inputs)
let converted conversion value = make (Operator (Conversion conversion)) [| value |]
let fence value waited = make Fence (Array.append [| value |] waited)
let i32 value = make (I32Constant (value land 0xFFFF_FFFF)) [||]

let distinct items =
  List.rev
    (List.fold_left (fun distinct item -> if List.mem item distinct then distinct else item :: distinct) [] items)

module Context = struct
  let contents context box_type =
    match TypeArena.definition context.types box_type with
    | Box contents -> contents
    | _ -> invalid_arg ("Expected a box, not " ^ TypeArena.describe context.types box_type ^ ".")

  let function_type context signature =
    TypeArena.intern context.types
      (Function ((context.erased :: signature.parameters) @ [ Type.unit ], signature.results @ [ Type.unit ]))

  let function_of context index = function_type context context.signatures.functions.(index)
  let erased_function types = TypeArena.intern types (Function ([], []))
  let closure context function_type = TypeArena.intern context.types (Sum [ [ function_type; context.erased ] ])

  (** A memory's or a table's box holds a structure of its array and the most it may grow to, so an imported one grows
      to its own maximum. *)
  let limited_box context array = context.boxes (TypeArena.intern context.types (Sum [ [ array; Type.i32 ] ]))

  let array_of context element = TypeArena.intern context.types (Array element)

  let field_type context (field : Field.t) =
    match field with
    | Function index when index < context.imported_functions -> closure context (function_of context index)
    | Function _ -> context.erased
    | Memory _ -> limited_box context Type.bytes
    | Data _ -> context.boxes Type.bytes
    | Table index -> limited_box context (array_of context context.signatures.tables.(index))
    | Element index -> context.boxes (array_of context context.signatures.elements.(index))
    | Global index -> context.boxes context.signatures.globals.(index)
    | Tag _ -> context.tag_box

  let thrown context tag =
    let tags = context.signatures.tags in
    Option.get (List.find_index (( = ) tags.(tag)) (distinct (Array.to_list tags)))
end

let limited context field array maximum =
  make
    (SumNew { sum_type = Context.contents context (Context.field_type context field); variant = 0 })
    [| array; maximum |]

let limited_parts context box =
  let contents = Context.contents context (Link.type_ context.types box) in
  RegionMatch.create context.types
    (result (BoxGet.create context.types box))
    (List.hd (TypeArena.variants context.types contents))
    (fun _ fields -> fields)

let held context (field : Field.t) box =
  match field with
  | Memory _ | Table _ -> Link.output (limited_parts context box) 0
  | Global _ -> box
  | Function _ | Data _ | Element _ | Tag _ -> result (BoxGet.create context.types box)

let zero value_type =
  match Type.scalar value_type with
  | Some `I32 -> make (I32Constant 0) [||]
  | Some `I64 -> make (I64Constant 0L) [||]
  | Some `F32 -> make (F32Constant 0l) [||]
  | Some `F64 -> make (F64Constant 0.0) [||]
  | Some (`I8 | `I16) | None -> make (SumNew { sum_type = value_type; variant = 0 }) [||]

let select types count tag values =
  let variant = Link.types types values in
  result (SumFromI32.create types (TypeArena.intern types (Sum (List.init count (fun _ -> variant)))) tag values)
