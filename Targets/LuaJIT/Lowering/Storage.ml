open IR
open Definitions
open Passes
open Numbers

let omission =
  Options.optimization "omit-luajit-bounds-checks" Storage Checks 3
    "Access without checking bounds, leaving an access out of bounds undefined."

let key index = add (to_number index) (constant 1.0)

(* LuaJIT: FFI accesses are unchecked, words are signed, and neither table.new nor ffi.new fills. *)
let length types array = extended (`LuaJITLength (Link.type_ types array)) [| array |]

(** Whether [count] elements from [start], numbers below 2^33, pass the array's end. *)
let beyond types array start count = less_than (to_number (length types array)) (add start count)

let bounded options types node outside access =
  ExtensionLowering.bounded types options omission node outside access (fun value_type ->
      extended (`LuaJITOutOfBounds value_type) [||])

(* LuaJIT: FFI accesses are unchecked, words are signed, and neither table.new nor ffi.new fills. *)

(** LuaJIT holds an I32 signed, so an index is checked as its unsigned number. *)
let element types inputs = beyond types inputs.(0) (unsigned inputs.(1)) (constant 1.0)

let get options types node =
  let bytes = Node.operand_type types node = Type.bytes in
  (* LuaJIT: FFI accesses are unchecked, words are signed, and neither table.new nor ffi.new fills. *)
  bounded options types node (element types) (fun inputs ->
      let array, index = (inputs.(0), inputs.(1)) in
      if bytes then of_number `I8 (extended `LuaJITReadU8 [| array; to_number index; constant 0.0 |])
      else Link.output (IR_LuaJIT.table_get types array (key index)) 0)

let set options types node =
  let bytes = Node.operand_type types node = Type.bytes in
  (* LuaJIT: FFI accesses are unchecked, words are signed, and neither table.new nor ffi.new fills. *)
  bounded options types node (element types) (fun inputs ->
      let array, index, value = (inputs.(0), inputs.(1), inputs.(2)) in
      if bytes then extended `LuaJITWriteU8 [| array; to_number index; constant 0.0; to_number value |]
      else Link.output (IR_LuaJIT.table_set types array (key index) value) 0)

let filled types array start count value =
  let loop fields =
    let carried = Link.types types fields in
    let step_type = RegionRepeat.step_type types carried carried in
    let repeat =
      RegionRepeat.create types fields carried (fun carried ->
          let key, last = (carried.(1), carried.(2)) in
          let set = Link.output (IR_LuaJIT.table_set types carried.(0) key carried.(3)) 0 in
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

let fill options types node =
  let bytes = Node.operand_type types node = Type.bytes in
  bounded options types node
    (* LuaJIT: FFI accesses are unchecked, words are signed, and neither table.new nor ffi.new fills. *)
    (fun inputs -> beyond types inputs.(0) (unsigned inputs.(1)) (unsigned inputs.(2)))
    (fun inputs ->
      if bytes then extended `LuaJITBytesFill inputs else filled types inputs.(0) inputs.(1) inputs.(2) inputs.(3))

let copy options types node =
  let array_type = Node.operand_type types node in
  let outside inputs =
    (* LuaJIT: FFI accesses are unchecked, words are signed, and neither table.new nor ffi.new fills. *)
    let count = unsigned inputs.(4) in
    Branch.either types
      (beyond types inputs.(0) (unsigned inputs.(1)) count)
      (beyond types inputs.(2) (unsigned inputs.(3)) count)
  in
  bounded options types node outside (fun inputs ->
      if array_type = Type.bytes then extended `LuaJITBytesCopy inputs
      else
        let last = add (to_number inputs.(3)) (to_number inputs.(4)) in
        extended (`LuaJITTableMove array_type) [| inputs.(0); key inputs.(1); inputs.(2); key inputs.(3); last |])

(* LuaJIT: FFI accesses are unchecked, words are signed, and neither table.new nor ffi.new fills. *)
let at offset inputs = [| inputs.(0); to_number inputs.(1); constant (float_of_int offset) |]

let accessed types offset value_type inputs =
  let start = add (unsigned inputs.(1)) (constant (float_of_int offset)) in
  beyond types inputs.(0) start (constant (float_of_int (Type.bits value_type / 8)))

let load options types node load =
  let offset = IR_Memory.offset load in
  bounded options types node
    (accessed types offset (Node.output_type types node 0))
    (fun inputs ->
      let at = at offset inputs in
      match load with
      | `MemoryLoadI8 _ -> of_number `I8 (extended `LuaJITReadU8 at)
      | `MemoryLoadI16 _ -> of_number `I16 (extended `LuaJITReadU16 at)
      (* LuaJIT: FFI accesses are unchecked, words are signed, and neither table.new nor ffi.new fills. *)
      | `MemoryLoadI32 _ -> of_number `I32 (extended `LuaJITReadI32 at)
      | `MemoryLoadI64 _ -> extended `LuaJITReadI64 at)

let store options types node store =
  let offset = IR_Memory.offset store in
  bounded options types node
    (accessed types offset (Link.type_ types node.inputs.(2)))
    (fun inputs ->
      let write operation value = extended operation (Array.append (at offset inputs) [| value |]) in
      let value = inputs.(2) in
      match store with
      (* LuaJIT: FFI accesses are unchecked, words are signed, and neither table.new nor ffi.new fills. *)
      | `MemoryStoreI8 _ -> write `LuaJITWriteU8 (to_number value)
      | `MemoryStoreI16 _ -> write `LuaJITWriteU16 (to_number value)
      | `MemoryStoreI32 _ -> write `LuaJITWriteI32 (to_number value)
      | `MemoryStoreI64 _ -> write `LuaJITWriteI64 value)

let allocate types allocation =
  let array_type = ArrayNew.array_type allocation in
  let bytes = array_type = Type.bytes in
  ExtensionLowering.allocation types allocation
    (* LuaJIT: FFI accesses are unchecked, words are signed, and neither table.new nor ffi.new fills. *)
    (fun length _ ->
      if bytes then extended `LuaJITBytesNew [| length |] else Link.output (IR_LuaJIT.array_new array_type length) 0)
    (fun array -> Link.output (IR_LuaJIT.allocated types array) 0)
    (fun fields ->
      (* LuaJIT: FFI accesses are unchecked, words are signed, and neither table.new nor ffi.new fills. *)
      let array, length, initial = (fields.(0), fields.(1), fields.(2)) in
      if bytes then extended `LuaJITBytesFill [| array; integer 0; length; initial |]
      else filled types array (integer 0) length initial)
