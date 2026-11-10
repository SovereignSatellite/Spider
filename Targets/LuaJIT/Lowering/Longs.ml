open IR
open Definitions
open Passes
open Numbers

(* LuaJIT: an I64 lowers to an FFI long. *)
let as_unsigned types value = cast (IR_LuaJIT.unsigned types) value

let through_unsigned types operation left right =
  cast Type.i64 (extended operation [| as_unsigned types left; as_unsigned types right |])

let binary types (operator : IntegerBinary.t) left right =
  (* LuaJIT: an I64 lowers to an FFI long. *)
  let both operation = extended operation [| left; right |] in
  match operator with
  | Add -> both `LuaJITAdd
  | Subtract -> both `LuaJITSubtract
  | Multiply -> both `LuaJITMultiply
  | DivideS -> both `LuaJITDivide
  | RemainderS -> both `LuaJITModulo
  | And -> both `LuaJITBitBand
  | Or -> both `LuaJITBitBor
  | ExclusiveOr -> both `LuaJITBitBxor
  | ShiftLeft -> both `LuaJITBitLshift
  | ShiftRightS -> both `LuaJITBitArshift
  | ShiftRightU -> both `LuaJITBitRshift
  | RotateLeft -> both `LuaJITBitRol
  | RotateRight -> both `LuaJITBitRor
  | DivideU -> through_unsigned types `LuaJITDivide left right
  | RemainderU -> through_unsigned types `LuaJITModulo left right

let narrow value = extended `LuaJITBitTobit [| value |]
let high value = narrow (extended `LuaJITBitRshift [| value; result (I64Constant 32L) [||] |])

let count (operator : IntegerCount.t) value =
  match operator with
  (* LuaJIT: an I64 lowers to an FFI long. *)
  | LeadingZeros | TrailingZeros -> None
  | CountOnes ->
      Some
        (to_long (add (to_number (Integers.count_ones (narrow value))) (to_number (Integers.count_ones (high value)))))

let compare types (operator : IntegerComparison.t) left right =
  match operator with
  (* LuaJIT: an I64 lowers to an FFI long. *)
  | Equal -> extended `LuaJITEqual [| left; right |]
  | LessThanS -> extended `LuaJITLessThan [| left; right |]
  | LessThanU -> extended `LuaJITLessThan [| as_unsigned types left; as_unsigned types right |]

let widen value = extended `LuaJITBitBand [| to_long value; result (I64Constant 0xFFFF_FFFFL) [||] |]
let sign_extend bits value = to_long (Integers.sign_extend bits (narrow value))
let convert_signed value = extended `LuaJITTonumber [| value |]
let convert_unsigned types value = convert_signed (as_unsigned types value)
let truncated types value = cast Type.i64 (as_unsigned types value)

(** A NaN's bits become those of LuaJIT's NaN of its sign before they are an F64, as LuaJIT reads another NaN as another
    value. *)
let canonical types bits =
  let long value = result (I64Constant value) [||] in
  let nan =
    extended `LuaJITLessThan [| long 0x7FF0_0000_0000_0000L; extended `LuaJITBitBand [| bits; long Int64.max_int |] |]
  in
  let of_sign fields =
    extended `LuaJITBitBor
      [| extended `LuaJITBitBand [| fields.(0); long Int64.min_int |]; long 0x7FF8_0000_0000_0000L |]
  in
  extended `LuaJITBitsIntoF64 [| Branch.choose_one types nan [| bits |] Type.i64 (fun fields -> fields.(0)) of_sign |]

let into_f64 types bits =
  match bits.node.kind with
  | Extension `LuaJITReadI64 ->
      let memory, address, offset = (bits.node.inputs.(0), bits.node.inputs.(1), bits.node.inputs.(2)) in
      let high = extended `LuaJITReadI32 [| memory; address; add offset (constant 4.0) |] in
      let masked word mask = to_number (extended `LuaJITBitBand [| word; constant mask |]) in
      let read fields = extended `LuaJITReadF64 (Array.sub fields 0 3) in
      let special fields =
        let low = extended `LuaJITReadI32 (Array.sub fields 0 3) in
        let fraction =
          Branch.either types
            (less_than (constant 0.0) (masked fields.(3) 1048575.0))
            (Branch.either types (less_than low (constant 0.0)) (less_than (constant 0.0) low))
        in
        Branch.choose_one types fraction fields Type.f64 read (fun fields ->
            Branch.choose_one types
              (less_than fields.(3) (constant 0.0))
              [||] Type.f64
              (fun _ -> constant Float.nan)
              (fun _ -> constant (Float.neg Float.nan)))
      in
      Branch.choose_one types
        (equal (masked high 2146435072.0) (constant 2146435072.0))
        [| memory; address; offset; high |] Type.f64 read special
  | _ -> canonical types bits
