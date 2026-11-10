open IR
open Definitions
open Passes
open Numbers

let omission =
  Options.optimization "omit-rust-bounds-checks" Storage Checks 3
    "Access without checking bounds, leaving an access out of bounds undefined."

let length types target array = cast target (Link.output (IR_Rust.length types array) 0)
let wide value = cast Type.i64 value
let long value = result (I64Constant (Int64.of_int value)) [||]

(** Whether [count] elements from [start], as I64s, pass the array's end. *)
let beyond types array start count =
  extended `RustLessThan [| length types Type.i64 array; extended `RustWrappingAdd [| start; count |] |]

let bounded options types node outside access =
  ExtensionLowering.bounded types options omission node outside access (fun value_type ->
      extended (`RustOutOfBounds value_type) [||])

let element types inputs = beyond types inputs.(0) (wide inputs.(1)) (long 1)

let get options types node =
  bounded options types node (element types) (fun inputs ->
      Link.output (IR_Rust.element_get types inputs.(0) (usize types inputs.(1))) 0)

let set options types node =
  bounded options types node (element types) (fun inputs ->
      Link.output (IR_Rust.element_set types inputs.(0) (usize types inputs.(1)) inputs.(2)) 0)

let size types node = length types Type.i32 node.inputs.(0)

let fill options types node =
  bounded options types node
    (fun inputs -> beyond types inputs.(0) (wide inputs.(1)) (wide inputs.(2)))
    (fun inputs ->
      Link.output (IR_Rust.element_fill types inputs.(0) (usize types inputs.(1)) (usize types inputs.(2)) inputs.(3)) 0)

let copy options types node =
  let outside inputs =
    let count = wide inputs.(4) in
    Branch.either types
      (beyond types inputs.(0) (wide inputs.(1)) count)
      (beyond types inputs.(2) (wide inputs.(3)) count)
  in
  bounded options types node outside (fun inputs ->
      let at, start, count = (usize types inputs.(1), usize types inputs.(3), usize types inputs.(4)) in
      Link.output (IR_Rust.element_copy types inputs.(0) at inputs.(2) start count) 0)

let at address offset = extended `RustWrappingAdd [| wide address; long offset |]

let accessed types offset value_type inputs =
  beyond types inputs.(0) (at inputs.(1) offset) (long (Type.bits value_type / 8))

let load options types node load =
  let offset, value_type = (IR_Memory.offset load, Node.output_type types node 0) in
  bounded options types node (accessed types offset value_type) (fun inputs ->
      Link.output (IR_Rust.read types inputs.(0) (usize types (at inputs.(1) offset)) value_type) 0)

let store options types node store =
  let offset = IR_Memory.offset store in
  bounded options types node
    (accessed types offset (Link.type_ types node.inputs.(2)))
    (fun inputs -> Link.output (IR_Rust.write types inputs.(0) (usize types (at inputs.(1) offset)) inputs.(2)) 0)

let allocate types allocation =
  ExtensionLowering.allocation types allocation
    (fun length initial -> Link.output (IR_Rust.array_new types length initial) 0)
    (fun array -> Link.output (IR_Rust.allocated types array) 0)
    (fun fields -> fields.(0))
