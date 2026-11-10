open IR

(* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
type TypeDefinition.extension += Thrown | Unsigned

let thrown types = TypeArena.extension types Thrown

(* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
let unsigned types = TypeArena.extension types Unsigned

let describe definition =
  match definition with
  (* LuaJIT: the FFI has unsigned longs. *)
  | Thrown -> "Thrown"
  | Unsigned -> "Unsigned"
  | _ -> invalid_arg "LuaJIT defines no such type."

type t =
  [ `LuaJITToNumber
  | `LuaJITFromNumber of Scalar.t
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    `LuaJITBitsIntoF64
  | `LuaJITBitsFromF64
  | `LuaJITBitsIntoF32
  | `LuaJITBitsFromF32
  | `LuaJITAdd
  | `LuaJITSubtract
  | `LuaJITMultiply
  | `LuaJITDivide
  | `LuaJITNegate
  | `LuaJITEqual
  | `LuaJITLessThan
  | `LuaJITLessThanEqual
  | `LuaJITModulo
  | `LuaJITMathFmod
  | `LuaJITMathAbs
  | `LuaJITMathFloor
  | `LuaJITMathCeil
  | `LuaJITMathSqrt
  | `LuaJITMathModf
  | `LuaJITBitBand
  | `LuaJITBitBor
  | `LuaJITBitBxor
  | `LuaJITBitLshift
  | `LuaJITBitRshift
  | `LuaJITBitArshift
  | `LuaJITBitRol
  | `LuaJITBitRor
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    `LuaJITBitTobit
  | `LuaJITCast of Type.t
  | `LuaJITTonumber
  | `LuaJITTableGet of Type.t * Type.t
  | `LuaJITTableSet of Type.t * Type.t
  | `LuaJITTableMove of Type.t
  | `LuaJITReadU8
  | `LuaJITReadU16
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    `LuaJITReadI32
  | `LuaJITReadI64
  | `LuaJITReadF64
  | `LuaJITWriteU8
  | `LuaJITWriteU16
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    `LuaJITWriteI32
  | `LuaJITWriteI64
  | `LuaJITWriteF64
  | `LuaJITArrayOf of Type.t * Type.t * string
  | `LuaJITLength of Type.t
  | `LuaJITBytesFromString of string
  | `LuaJITBytesFill
  | `LuaJITBytesCopy
  | `LuaJITBoxNew of Type.t
  | `LuaJITBoxGet of Type.t
  | `LuaJITBoxSet of Type.t
  | `LuaJITRetype of Type.t
  | `LuaJITArrayNew of Type.t
  | `LuaJITBytesNew
  | `LuaJITAllocated of Type.t
  | `LuaJITTry
  | `LuaJITCaught of Type.t list
  | `LuaJITOutOfBounds of Type.t ]

let name node =
  match node with
  | `LuaJITToNumber -> "LuaJIT.ToNumber"
  | `LuaJITFromNumber _ -> "LuaJIT.FromNumber"
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITBitsIntoF64 -> "LuaJIT.Bits.IntoF64"
  | `LuaJITBitsFromF64 -> "LuaJIT.Bits.FromF64"
  | `LuaJITBitsIntoF32 -> "LuaJIT.Bits.IntoF32"
  | `LuaJITBitsFromF32 -> "LuaJIT.Bits.FromF32"
  | `LuaJITAdd -> "LuaJIT.Add"
  | `LuaJITSubtract -> "LuaJIT.Subtract"
  | `LuaJITMultiply -> "LuaJIT.Multiply"
  | `LuaJITDivide -> "LuaJIT.Divide"
  | `LuaJITNegate -> "LuaJIT.Negate"
  | `LuaJITEqual -> "LuaJIT.Equal"
  | `LuaJITLessThan -> "LuaJIT.LessThan"
  | `LuaJITLessThanEqual -> "LuaJIT.LessThanEqual"
  | `LuaJITModulo -> "LuaJIT.Modulo"
  | `LuaJITMathFmod -> "LuaJIT.Math.Fmod"
  | `LuaJITMathAbs -> "LuaJIT.Math.Abs"
  | `LuaJITMathFloor -> "LuaJIT.Math.Floor"
  | `LuaJITMathCeil -> "LuaJIT.Math.Ceil"
  | `LuaJITMathSqrt -> "LuaJIT.Math.Sqrt"
  | `LuaJITMathModf -> "LuaJIT.Math.Modf"
  | `LuaJITBitBand -> "LuaJIT.Bit.Band"
  | `LuaJITBitBor -> "LuaJIT.Bit.Bor"
  | `LuaJITBitBxor -> "LuaJIT.Bit.Bxor"
  | `LuaJITBitLshift -> "LuaJIT.Bit.Lshift"
  | `LuaJITBitRshift -> "LuaJIT.Bit.Rshift"
  | `LuaJITBitArshift -> "LuaJIT.Bit.Arshift"
  | `LuaJITBitRol -> "LuaJIT.Bit.Rol"
  | `LuaJITBitRor -> "LuaJIT.Bit.Ror"
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITBitTobit -> "LuaJIT.Bit.Tobit"
  | `LuaJITCast _ -> "LuaJIT.Cast"
  | `LuaJITTonumber -> "LuaJIT.Tonumber"
  | `LuaJITTableGet _ -> "LuaJIT.Table.Get"
  | `LuaJITTableSet _ -> "LuaJIT.Table.Set"
  | `LuaJITTableMove _ -> "LuaJIT.Table.Move"
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITReadU8 -> "LuaJIT.ReadU8"
  | `LuaJITReadU16 -> "LuaJIT.ReadU16"
  | `LuaJITReadI32 -> "LuaJIT.ReadI32"
  | `LuaJITReadI64 -> "LuaJIT.ReadI64"
  | `LuaJITReadF64 -> "LuaJIT.ReadF64"
  | `LuaJITWriteU8 -> "LuaJIT.WriteU8"
  | `LuaJITWriteU16 -> "LuaJIT.WriteU16"
  | `LuaJITWriteI32 -> "LuaJIT.WriteI32"
  | `LuaJITWriteI64 -> "LuaJIT.WriteI64"
  | `LuaJITWriteF64 -> "LuaJIT.WriteF64"
  | `LuaJITArrayOf _ -> "LuaJIT.ArrayOf"
  | `LuaJITLength _ -> "LuaJIT.Length"
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITBytesFromString _ -> "LuaJIT.BytesFromString"
  | `LuaJITBytesFill -> "LuaJIT.Bytes.Fill"
  | `LuaJITBytesCopy -> "LuaJIT.Bytes.Copy"
  | `LuaJITBoxNew _ -> "LuaJIT.Box.New"
  | `LuaJITBoxGet _ -> "LuaJIT.Box.Get"
  | `LuaJITBoxSet _ -> "LuaJIT.Box.Set"
  | `LuaJITRetype _ -> "LuaJIT.Retype"
  | `LuaJITArrayNew _ -> "LuaJIT.ArrayNew"
  | `LuaJITBytesNew -> "LuaJIT.BytesNew"
  | `LuaJITAllocated _ -> "LuaJIT.Allocated"
  | `LuaJITTry -> "LuaJIT.Try"
  | `LuaJITCaught _ -> "LuaJIT.Caught"
  | `LuaJITOutOfBounds _ -> "LuaJIT.OutOfBounds"

let try_outputs types node =
  Type.boolean :: thrown types :: snd (TypeArena.signature types (Node.operand_type types node))

let output_count types node extended =
  match extended with
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITToNumber | `LuaJITFromNumber _ | `LuaJITBitsIntoF64 | `LuaJITBitsFromF64 | `LuaJITBitsIntoF32
  | `LuaJITBitsFromF32 | `LuaJITAdd | `LuaJITSubtract | `LuaJITMultiply | `LuaJITDivide | `LuaJITNegate | `LuaJITEqual
  | `LuaJITLessThan | `LuaJITLessThanEqual | `LuaJITModulo | `LuaJITMathFmod | `LuaJITMathAbs | `LuaJITMathFloor
  | `LuaJITMathCeil | `LuaJITMathSqrt | `LuaJITMathModf | `LuaJITBitBand | `LuaJITBitBor | `LuaJITBitBxor
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITBitLshift | `LuaJITBitRshift | `LuaJITBitArshift | `LuaJITBitRol | `LuaJITBitRor | `LuaJITBitTobit
  | `LuaJITCast _ | `LuaJITTonumber | `LuaJITTableGet _ | `LuaJITTableSet _ | `LuaJITTableMove _ | `LuaJITReadU8
  | `LuaJITReadU16 | `LuaJITReadI32 | `LuaJITReadI64 | `LuaJITReadF64 | `LuaJITWriteU8 | `LuaJITWriteU16
  | `LuaJITWriteI32 | `LuaJITWriteI64 | `LuaJITWriteF64 | `LuaJITArrayOf _ | `LuaJITLength _ | `LuaJITBytesFromString _
  | `LuaJITBytesFill | `LuaJITBytesCopy | `LuaJITBoxNew _ | `LuaJITBoxGet _ | `LuaJITBoxSet _ | `LuaJITRetype _
  | `LuaJITArrayNew _ | `LuaJITBytesNew | `LuaJITAllocated _ | `LuaJITOutOfBounds _ ->
      1
  | `LuaJITTry -> List.length (try_outputs types node)
  | `LuaJITCaught payload_types -> List.length payload_types

(* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)

(** An operator takes numbers, and I64s or [Unsigned]s where it has an integer form. *)
let integer_or_number types node =
  Node.admitted types node (fun operand -> operand = Type.i64 || operand = unsigned types) Type.f64

let long_or_number types node = Node.admitted types node (( = ) Type.i64) Type.f64

let output_type types node extended port =
  match extended with
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITToNumber | `LuaJITBitsIntoF64 | `LuaJITBitsIntoF32 | `LuaJITNegate | `LuaJITMathFmod | `LuaJITMathAbs
  | `LuaJITMathFloor | `LuaJITMathCeil | `LuaJITMathSqrt | `LuaJITMathModf | `LuaJITTonumber | `LuaJITReadU8
  | `LuaJITReadU16 | `LuaJITReadI32 | `LuaJITReadF64 ->
      Type.f64
  | `LuaJITFromNumber result -> Type.of_scalar result
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITBitsFromF64 | `LuaJITReadI64 -> Type.i64
  | `LuaJITBitsFromF32 -> Type.f32
  | `LuaJITAdd | `LuaJITSubtract | `LuaJITMultiply | `LuaJITDivide | `LuaJITModulo -> integer_or_number types node
  | `LuaJITEqual | `LuaJITLessThan | `LuaJITLessThanEqual | `LuaJITAllocated _ -> Type.boolean
  | `LuaJITBitBand | `LuaJITBitBor | `LuaJITBitBxor | `LuaJITBitLshift | `LuaJITBitRshift | `LuaJITBitArshift
  | `LuaJITBitRol | `LuaJITBitRor ->
      if long_or_number types node = Type.i64 then Type.i64 else Type.i32
  | `LuaJITBitTobit | `LuaJITLength _ -> Type.i32
  | `LuaJITCast target | `LuaJITRetype target -> target
  | `LuaJITTableGet (element_type, _) | `LuaJITOutOfBounds element_type -> element_type
  | `LuaJITTableSet (array_type, _) | `LuaJITTableMove array_type | `LuaJITArrayNew array_type -> array_type
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITWriteU8 | `LuaJITWriteU16 | `LuaJITWriteI32 | `LuaJITWriteI64 | `LuaJITWriteF64 | `LuaJITBytesFromString _
  | `LuaJITBytesFill | `LuaJITBytesCopy | `LuaJITBytesNew ->
      Type.bytes
  | `LuaJITArrayOf (array_type, _, _) | `LuaJITBoxNew array_type | `LuaJITBoxSet array_type -> array_type
  | `LuaJITBoxGet box_type -> TypeArena.contents types box_type
  | `LuaJITTry -> List.nth (try_outputs types node) port
  | `LuaJITCaught payload_types -> List.nth payload_types port

let input_type types node extended index =
  match extended with
  | `LuaJITToNumber ->
      (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
      Node.admitted types node (fun operand -> Option.is_some (Type.scalar operand) && operand <> Type.i64) Type.f64
  | `LuaJITFromNumber _ | `LuaJITBitsFromF64 | `LuaJITBitsFromF32 | `LuaJITNegate | `LuaJITLessThanEqual
  | `LuaJITMathFmod | `LuaJITMathAbs | `LuaJITMathFloor | `LuaJITMathCeil | `LuaJITMathSqrt | `LuaJITMathModf ->
      Type.f64
  | `LuaJITBitsIntoF64 -> Type.i64
  | `LuaJITBitsIntoF32 -> Type.f32
  | `LuaJITAdd | `LuaJITSubtract | `LuaJITMultiply | `LuaJITDivide | `LuaJITLessThan | `LuaJITModulo | `LuaJITCast _ ->
      integer_or_number types node
  | `LuaJITEqual -> Node.operand_type types node
  | `LuaJITBitBand | `LuaJITBitBor | `LuaJITBitBxor | `LuaJITBitLshift | `LuaJITBitRshift | `LuaJITBitArshift
  | `LuaJITBitRol | `LuaJITBitRor | `LuaJITBitTobit ->
      (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
      long_or_number types node
  | `LuaJITTonumber -> Node.admitted types node (( = ) (unsigned types)) Type.i64
  | `LuaJITTableGet (_, array_type) -> if index = 0 then array_type else Type.f64
  | `LuaJITTableSet (array_type, value_type) -> ( match index with 0 -> array_type | 1 -> Type.f64 | _ -> value_type)
  | `LuaJITTableMove array_type -> ( match index with 0 | 2 -> array_type | _ -> Type.f64)
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITReadU8 | `LuaJITReadU16 | `LuaJITReadI32 | `LuaJITReadI64 | `LuaJITReadF64 | `LuaJITWriteU8 | `LuaJITWriteU16
  | `LuaJITWriteI32 | `LuaJITWriteF64 ->
      if index = 0 then Type.bytes else Type.f64
  | `LuaJITWriteI64 -> ( match index with 0 -> Type.bytes | 3 -> Type.i64 | _ -> Type.f64)
  | `LuaJITArrayOf _ | `LuaJITBytesFromString _ | `LuaJITOutOfBounds _ ->
      invalid_arg "LuaJIT.OutOfBounds, ArrayOf and BytesFromString read nothing."
  | `LuaJITLength array_type | `LuaJITAllocated array_type -> array_type
  | `LuaJITBytesFill -> ( match index with 0 -> Type.bytes | 3 -> Type.i8 | _ -> Type.i32)
  | `LuaJITBytesCopy -> ( match index with 0 | 2 -> Type.bytes | _ -> Type.i32)
  | `LuaJITBoxNew box_type -> TypeArena.contents types box_type
  | `LuaJITBoxGet box_type -> box_type
  | `LuaJITBoxSet box_type -> if index = 0 then box_type else TypeArena.contents types box_type
  | `LuaJITRetype _ -> Node.operand_type types node
  | `LuaJITArrayNew _ | `LuaJITBytesNew -> Type.i32
  | `LuaJITTry ->
      let callee = Node.operand_type types node in
      List.nth (callee :: fst (TypeArena.signature types callee)) index
  | `LuaJITCaught _ -> thrown types

let base node : Definitions.base =
  match node with
  | `LuaJITToNumber | `LuaJITFromNumber _ -> ViewOperation
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITBitsIntoF64 | `LuaJITBitsFromF64 | `LuaJITBitsIntoF32 | `LuaJITBitsFromF32 | `LuaJITNegate | `LuaJITMathAbs
  | `LuaJITMathFloor | `LuaJITMathCeil | `LuaJITMathSqrt | `LuaJITMathModf | `LuaJITBitTobit | `LuaJITCast _
  | `LuaJITTonumber | `LuaJITLength _ | `LuaJITRetype _ ->
      UnaryOperation
  | `LuaJITAdd | `LuaJITSubtract | `LuaJITMultiply | `LuaJITDivide | `LuaJITEqual | `LuaJITLessThan
  | `LuaJITLessThanEqual | `LuaJITModulo | `LuaJITMathFmod | `LuaJITBitBand | `LuaJITBitBor | `LuaJITBitBxor
  | `LuaJITBitLshift | `LuaJITBitRshift | `LuaJITBitArshift | `LuaJITBitRol | `LuaJITBitRor ->
      BinaryOperation
  | `LuaJITTableGet _ | `LuaJITReadU8 | `LuaJITReadU16 | `LuaJITReadI32 | `LuaJITReadI64 | `LuaJITReadF64
  | `LuaJITArrayOf _ | `LuaJITBytesFromString _ | `LuaJITBoxNew _ | `LuaJITBoxGet _ | `LuaJITArrayNew _
  | `LuaJITBytesNew | `LuaJITAllocated _ | `LuaJITTry | `LuaJITCaught _ | `LuaJITOutOfBounds _ ->
      Operation
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITTableSet _ | `LuaJITTableMove _ | `LuaJITWriteU8 | `LuaJITWriteU16 | `LuaJITWriteI32 | `LuaJITWriteI64
  | `LuaJITWriteF64 | `LuaJITBytesFill | `LuaJITBytesCopy | `LuaJITBoxSet _ ->
      Write

let holds node =
  match node with
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITToNumber | `LuaJITFromNumber _ | `LuaJITBitsIntoF64 | `LuaJITBitsFromF64 | `LuaJITBitsIntoF32
  | `LuaJITBitsFromF32 | `LuaJITAdd | `LuaJITSubtract | `LuaJITMultiply | `LuaJITDivide | `LuaJITNegate | `LuaJITModulo
  | `LuaJITMathFmod | `LuaJITMathAbs | `LuaJITMathFloor | `LuaJITMathCeil | `LuaJITMathSqrt | `LuaJITMathModf
  | `LuaJITBitBand | `LuaJITBitBor | `LuaJITBitBxor | `LuaJITBitLshift | `LuaJITBitRshift | `LuaJITBitArshift
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITBitRol | `LuaJITBitRor | `LuaJITBitTobit | `LuaJITCast _ | `LuaJITTonumber | `LuaJITTableGet _
  | `LuaJITTableSet _ | `LuaJITTableMove _ | `LuaJITReadU8 | `LuaJITReadU16 | `LuaJITReadI32 | `LuaJITReadI64
  | `LuaJITReadF64 | `LuaJITWriteU8 | `LuaJITWriteU16 | `LuaJITWriteI32 | `LuaJITWriteI64 | `LuaJITWriteF64
  | `LuaJITArrayOf _ | `LuaJITLength _ | `LuaJITBytesFromString _ | `LuaJITBytesFill | `LuaJITBytesCopy
  | `LuaJITBoxNew _ | `LuaJITBoxGet _ | `LuaJITBoxSet _ | `LuaJITRetype _ | `LuaJITArrayNew _ | `LuaJITBytesNew
  | `LuaJITAllocated _ | `LuaJITTry | `LuaJITCaught _ | `LuaJITOutOfBounds _ ->
      0
  | `LuaJITEqual -> Outcomes.equal
  | `LuaJITLessThan -> Outcomes.less
  | `LuaJITLessThanEqual -> Outcomes.less lor Outcomes.equal

let commutative node =
  match node with
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITToNumber | `LuaJITFromNumber _ | `LuaJITBitsIntoF64 | `LuaJITBitsFromF64 | `LuaJITBitsIntoF32
  | `LuaJITBitsFromF32 | `LuaJITSubtract | `LuaJITDivide | `LuaJITNegate | `LuaJITLessThan | `LuaJITLessThanEqual
  | `LuaJITModulo | `LuaJITMathFmod | `LuaJITMathAbs | `LuaJITMathFloor | `LuaJITMathCeil | `LuaJITMathSqrt
  | `LuaJITMathModf | `LuaJITBitLshift | `LuaJITBitRshift | `LuaJITBitArshift | `LuaJITBitRol | `LuaJITBitRor
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITBitTobit | `LuaJITCast _ | `LuaJITTonumber | `LuaJITTableGet _ | `LuaJITTableSet _ | `LuaJITTableMove _
  | `LuaJITReadU8 | `LuaJITReadU16 | `LuaJITReadI32 | `LuaJITReadI64 | `LuaJITReadF64 | `LuaJITWriteU8 | `LuaJITWriteU16
  | `LuaJITWriteI32 | `LuaJITWriteI64 | `LuaJITWriteF64 | `LuaJITArrayOf _ | `LuaJITLength _ | `LuaJITBytesFromString _
  | `LuaJITBytesFill | `LuaJITBytesCopy | `LuaJITBoxNew _ | `LuaJITBoxGet _ | `LuaJITBoxSet _ | `LuaJITRetype _
  | `LuaJITArrayNew _ | `LuaJITBytesNew | `LuaJITAllocated _ | `LuaJITTry | `LuaJITCaught _ | `LuaJITOutOfBounds _ ->
      false
  | `LuaJITAdd | `LuaJITMultiply | `LuaJITEqual | `LuaJITBitBand | `LuaJITBitBor | `LuaJITBitBxor -> true

let unrepeatable node =
  match node with
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITToNumber | `LuaJITFromNumber _ | `LuaJITBitsIntoF64 | `LuaJITBitsFromF64 | `LuaJITBitsIntoF32
  | `LuaJITBitsFromF32 | `LuaJITAdd | `LuaJITSubtract | `LuaJITMultiply | `LuaJITDivide | `LuaJITNegate | `LuaJITEqual
  | `LuaJITLessThan | `LuaJITLessThanEqual | `LuaJITModulo | `LuaJITMathFmod | `LuaJITMathAbs | `LuaJITMathFloor
  | `LuaJITMathCeil | `LuaJITMathSqrt | `LuaJITMathModf | `LuaJITBitBand | `LuaJITBitBor | `LuaJITBitBxor
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  | `LuaJITBitLshift | `LuaJITBitRshift | `LuaJITBitArshift | `LuaJITBitRol | `LuaJITBitRor | `LuaJITBitTobit
  | `LuaJITCast _ | `LuaJITTonumber | `LuaJITTableGet _ | `LuaJITTableSet _ | `LuaJITReadU8 | `LuaJITReadU16
  | `LuaJITReadI32 | `LuaJITReadI64 | `LuaJITReadF64 | `LuaJITWriteU8 | `LuaJITWriteU16 | `LuaJITWriteI32
  | `LuaJITWriteI64 | `LuaJITWriteF64 | `LuaJITLength _ | `LuaJITBytesFill | `LuaJITBoxGet _ | `LuaJITBoxSet _
  | `LuaJITRetype _ | `LuaJITAllocated _ | `LuaJITCaught _ | `LuaJITOutOfBounds _ ->
      false
  | `LuaJITTableMove _ | `LuaJITArrayOf _ | `LuaJITBytesFromString _ | `LuaJITBytesCopy | `LuaJITBoxNew _
  | `LuaJITArrayNew _ | `LuaJITBytesNew | `LuaJITTry ->
      true

let array_access _ = None

let table_get types array key =
  let array_type = Link.type_ types array in
  Node.create (Extension (`LuaJITTableGet (TypeArena.element types array_type, array_type))) [| array; key |]

let table_set types array key value =
  Node.create (Extension (`LuaJITTableSet (Link.type_ types array, Link.type_ types value))) [| array; key; value |]

(* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
let array_new array_type length = Node.create (Extension (`LuaJITArrayNew array_type)) [| length |]
let allocated types array = Node.create (Extension (`LuaJITAllocated (Link.type_ types array))) [| array |]
let try_ callee arguments = Node.create (Extension `LuaJITTry) (Array.append [| callee |] arguments)
let caught thrown payload_types = Node.create (Extension (`LuaJITCaught payload_types)) [| thrown |]
