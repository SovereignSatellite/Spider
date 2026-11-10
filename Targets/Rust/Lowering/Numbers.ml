open IR
open Definitions

let result kind inputs = Link.output (Node.create kind inputs) 0
let extended operation inputs = result (Extension operation) inputs
let cast target value = extended (`RustCast target) [| value |]
let signed_of types width = IR_Rust.signed types (Scalar.bits width)
let usize types value = cast (IR_Rust.usize types) value
