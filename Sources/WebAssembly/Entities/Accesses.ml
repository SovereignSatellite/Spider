open IR
open Entities

let read context state field reading =
  let value = reading (State.contents context state field Reads) in
  ([ value ], State.after state field Reads value)

let write context state field writing =
  ([], State.after state field Writes (writing (State.contents context state field Writes)))

let copy context state destination source operands =
  let into = State.contents context state destination Writes in
  let operands = Array.of_list operands in
  let copied from = result (Write.array_copy context.types into operands.(0) from operands.(1) operands.(2)) in
  if Field.compare source destination = 0 then ([], State.after state destination Writes (copied into))
  else
    let written = copied (State.contents context state source Reads) in
    ([], State.after (State.after state destination Writes written) source Reads written)

let fill context state field operands =
  let operands = Array.of_list operands in
  write context state field (fun array ->
      result (Write.array_fill context.types array operands.(0) operands.(2) operands.(1)))

let empty context state field array =
  let held = State.contents context state field Writes in
  let set = result (Write.box_set context.types (fence (State.box state field) [| held |]) array) in
  let finalized = fence set [| result (ArrayFinalize.create context.types held) |] in
  ([], State.replaced state field finalized (fence array [| finalized |]))

let set context state field value =
  let written = result (Write.box_set context.types (State.box state field) value) in
  State.replaced state field written (held context field written)
