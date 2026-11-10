open IR
open Definitions
open Passes
open Numbers

let omission =
  Options.optimization "omit-luau-bounds-checks" Storage Checks 3
    "Access without checking bounds, leaving an access out of bounds undefined."

let key index = add (to_number index) (constant 1.0)

let length types array =
  (* Luau: buffers check their own bounds and have their own operations, words are unsigned, and table.create fills. *)
  let array_type = Link.type_ types array in
  extended (if array_type = Type.bytes then `LuauBufferLen else `LuauLength array_type) [| array |]

(** Whether [count] elements from [start], numbers below 2^33, pass the array's end. *)
let beyond types array start count = less_than (to_number (length types array)) (add start count)

(* Luau: buffers check their own bounds and have their own operations, words are unsigned, and table.create fills. *)

(** Luau's buffer accesses check their own bounds, so only table accesses come here. *)
let bounded options types node outside access =
  ExtensionLowering.bounded types options omission node outside access (fun value_type ->
      extended (`LuauOutOfBounds value_type) [||])

(* Luau: buffers check their own bounds and have their own operations, words are unsigned, and table.create fills. *)
let element types inputs = beyond types inputs.(0) (to_number inputs.(1)) (constant 1.0)

let get options types node =
  let bytes = Node.operand_type types node = Type.bytes in
  (* Luau: buffers check their own bounds and have their own operations, words are unsigned, and table.create fills. *)
  let array, index = (node.inputs.(0), node.inputs.(1)) in
  if bytes then of_number `I8 (extended `LuauBufferReadU8 [| array; to_number index |])
  else
    (* Luau: buffers check their own bounds and have their own operations, words are unsigned, and table.create fills. *)
    bounded options types node (element types) (fun inputs ->
        Link.output (IR_Luau.table_get types inputs.(0) (key inputs.(1))) 0)

let set options types node =
  let bytes = Node.operand_type types node = Type.bytes in
  (* Luau: buffers check their own bounds and have their own operations, words are unsigned, and table.create fills. *)
  let array, index, value = (node.inputs.(0), node.inputs.(1), node.inputs.(2)) in
  if bytes then extended `LuauBufferWriteU8 [| array; to_number index; to_number value |]
  else
    (* Luau: buffers check their own bounds and have their own operations, words are unsigned, and table.create fills. *)
    bounded options types node (element types) (fun inputs ->
        Link.output (IR_Luau.table_set types inputs.(0) (key inputs.(1)) inputs.(2)) 0)

let filled types array start count value =
  let loop fields =
    let carried = Link.types types fields in
    let step_type = RegionRepeat.step_type types carried carried in
    let repeat =
      RegionRepeat.create types fields carried (fun carried ->
          let key, last = (carried.(1), carried.(2)) in
          let set = Link.output (IR_Luau.table_set types carried.(0) key carried.(3)) 0 in
          let next = [| set; add key (constant 1.0); last; carried.(3) |] in
          Link.output (SumFromI32.create types step_type (result I32FromSum [| less_than key last |]) next) 0)
    in
    Link.output repeat 0
  in
  let last = add (to_number start) (to_number count) in
  Branch.choose_one types
    (less_than (constant 0.0) (to_number count))
    [| array; key start; last; value |]
    (Link.type_ types array)
    (fun fields -> fields.(0))
    loop

(* Luau: buffers check their own bounds and have their own operations, words are unsigned, and table.create fills. *)
let buffer_fill buffer start count value = extended `LuauBufferFill [| buffer; start; value; count |]

let fill options types node =
  let bytes = Node.operand_type types node = Type.bytes in
  let inputs = node.inputs in
  if bytes then buffer_fill inputs.(0) inputs.(1) inputs.(2) inputs.(3)
  else
    bounded options types node
      (fun inputs -> beyond types inputs.(0) (to_number inputs.(1)) (to_number inputs.(2)))
      (fun inputs -> filled types inputs.(0) inputs.(1) inputs.(2) inputs.(3))

let copy options types node =
  let array_type = Node.operand_type types node in
  let outside inputs =
    (* Luau: buffers check their own bounds and have their own operations, words are unsigned, and table.create fills. *)
    let count = to_number inputs.(4) in
    Branch.either types
      (beyond types inputs.(0) (to_number inputs.(1)) count)
      (beyond types inputs.(2) (to_number inputs.(3)) count)
  in
  if array_type = Type.bytes then extended `LuauBufferCopy node.inputs
  else
    bounded options types node outside (fun inputs ->
        let last = add (to_number inputs.(3)) (to_number inputs.(4)) in
        extended (`LuauTableMove array_type) [| inputs.(0); key inputs.(1); inputs.(2); key inputs.(3); last |])

(* Luau: buffers check their own bounds and have their own operations, words are unsigned, and table.create fills. *)

(** The sum is exact, so where it is past the memory's end the access is undefined. *)
let offset address offset = add (to_number address) (constant (float_of_int offset))

let load node load =
  let memory, at = (node.inputs.(0), offset node.inputs.(1) (IR_Memory.offset load)) in
  match load with
  | `MemoryLoadI8 _ -> of_number `I8 (extended `LuauBufferReadU8 [| memory; at |])
  | `MemoryLoadI16 _ -> of_number `I16 (extended `LuauBufferReadU16 [| memory; at |])
  | `MemoryLoadI32 _ -> of_number `I32 (extended `LuauBufferReadU32 [| memory; at |])
  | `MemoryLoadI64 _ -> of_number `I64 (extended `LuauBufferReadF64 [| memory; at |])

let store node store =
  (* Luau: buffers check their own bounds and have their own operations, words are unsigned, and table.create fills. *)
  let memory, at = (node.inputs.(0), offset node.inputs.(1) (IR_Memory.offset store)) in
  let value = to_number node.inputs.(2) in
  match store with
  | `MemoryStoreI8 _ -> extended `LuauBufferWriteU8 [| memory; at; value |]
  | `MemoryStoreI16 _ -> extended `LuauBufferWriteU16 [| memory; at; value |]
  | `MemoryStoreI32 _ -> extended `LuauBufferWriteU32 [| memory; at; value |]
  | `MemoryStoreI64 _ -> extended `LuauBufferWriteF64 [| memory; at; value |]

let allocate types allocation =
  let array_type = ArrayNew.array_type allocation in
  let bytes = array_type = Type.bytes in
  ExtensionLowering.allocation types allocation
    (* Luau: buffers check their own bounds and have their own operations, words are unsigned, and table.create fills. *)
    (fun length initial ->
      if bytes then extended `LuauBufferCreate [| length |] else Link.output (IR_Luau.array_new types length initial) 0)
    (fun array -> Link.output (IR_Luau.allocated types array) 0)
    (fun fields ->
      (* Luau: buffers check their own bounds and have their own operations, words are unsigned, and table.create fills. *)
      if bytes && Constant.bits allocation.inputs.(1) <> Some 0L then
        buffer_fill fields.(0) (integer 0) fields.(1) fields.(2)
      else fields.(0))
