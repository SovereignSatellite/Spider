open IR
open Definitions
open Entities
open Traps

let failed box current = [| i32 0xFFFF_FFFF; box; current |]

let replace_grown context allocation field allocated inside =
  let current = inside.(1) in
  if allocated = 0 then failed inside.(3) current
  else
    let zero = i32 0 in
    let array = result (traps_in context inside.(0) 0 [ ArrayNew.array_type allocation ] Fun.id) in
    let copied = result (Write.array_copy context.types array zero current zero (make ArraySize [| current |])) in
    let held = result (Write.box_set context.types inside.(3) (limited context field copied inside.(4))) in
    let released = result (ArrayFinalize.create context.types (fence current [| copied |])) in
    [| inside.(2); fence held [| released |]; copied |]

let grown context field box current length variant values =
  if variant = 1 then failed values.(3) values.(0)
  else
    let allocation =
      ArrayNew.create context.types
        (length (make (Operator (IntegerBinary (`I32, Add))) [| values.(1); values.(2) |]))
        values.(5)
    in
    let allocated = make I32FromSum [| result allocation |] in
    let replaced =
      RegionMatch.create context.types
        (select context.types 2 allocated [| result allocation; values.(0); values.(1); values.(3); values.(4) |])
        [ Type.i32; Link.type_ context.types box; Link.type_ context.types current ]
        (replace_grown context allocation field)
    in
    Array.init 3 (Link.output replaced)

let grow context state field units delta element length =
  let maximum = Link.output (limited_parts context (State.item state field)) 1 in
  let current = State.contents context state field Writes in
  let size = units current in
  let box = State.box state field in
  let fails =
    make I32FromSum
      [|
        make
          (Operator (IntegerComparison (`I32, LessThanU)))
          [| make (Operator (IntegerBinary (`I32, Subtract))) [| maximum; size |]; delta |];
      |]
  in
  let grown =
    RegionMatch.create context.types
      (select context.types 2 fails [| current; size; delta; box; maximum; element |])
      [ Type.i32; Link.type_ context.types box; Link.type_ context.types current ]
      (grown context field box current length)
  in
  ([ Link.output grown 0 ], State.replaced state field (Link.output grown 1) (Link.output grown 2))
