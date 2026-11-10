open IR
open Definitions
open Types
open Entities
open Traps
open Accesses

let references context element items =
  Seq.fold_lefti
    (fun array index item -> result (Write.array_set context.types array (i32 index) item))
    (allocate context (i32 (List.length items)) (zero element))
    (List.to_seq items)

let table index : Field.t = Table (Types.index index)
let is_null reference = make (Operator (IntegerBinary (`I32, ExclusiveOr))) [| make I32FromSum [| reference |]; i32 1 |]

let get context state index operands =
  read context state (table index) (fun elements -> result (ArrayGet.create context.types elements (List.hd operands)))

let set context state index operands =
  let operands = Array.of_list operands in
  write context state (table index) (fun elements ->
      result (Write.array_set context.types elements operands.(0) operands.(1)))

let size context state index = read context state (table index) (fun elements -> make ArraySize [| elements |])

let grow context state index operands =
  Growth.grow context state (table index)
    (fun elements -> make ArraySize [| elements |])
    (List.nth operands 1) (List.hd operands) Fun.id

let drop context state segment =
  empty context state (Element segment) (references context context.signatures.elements.(segment) [])

let non_null context reference =
  let checked = select context.types 2 (make I32FromSum [| reference |]) [| reference |] in
  result (traps_in context checked 0 [ Link.type_ context.types reference ] Fun.id)

let call_reference context state type_index operands =
  let signature = Lazy.force context.signatures.types.(Types.index type_index) in
  let expected = Context.function_type context signature in
  let arguments = List.length operands - 1 in
  let opened =
    traps_in context (List.nth operands arguments) 0 [ expected; context.erased ] (fun parts ->
        [| make (FunctionRetype expected) [| parts.(0) |]; parts.(1) |])
  in
  Calls.apply context state (Indirect signature) (Link.output opened 0) (Link.output opened 1)
    (List.take arguments operands)

let call_indirect context state type_index index operands =
  let arguments = List.length operands - 1 in
  let references, state = get context state index [ List.nth operands arguments ] in
  call_reference context state type_index (List.take arguments operands @ references)
