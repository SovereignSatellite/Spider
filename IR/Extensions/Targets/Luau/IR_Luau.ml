open IR

type TypeDefinition.extension += Thrown

let thrown types = TypeArena.extension types Thrown
let describe definition = match definition with Thrown -> "Thrown" | _ -> invalid_arg "Luau defines no such type."

type t =
  [ `LuauToNumber
  | `LuauFromNumber of Scalar.t
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    `LuauBitsFromI64
  | `LuauBitsLowFromI64
  | `LuauBitsHighFromI64
  | `LuauBitsIntoI64
  | `LuauBitsIntoF32
  | `LuauBitsFromF32
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    `LuauBitsRoundF32
  | `LuauAdd
  | `LuauSubtract
  | `LuauMultiply
  | `LuauDivide
  | `LuauNegate
  | `LuauEqual
  | `LuauLessThan
  | `LuauLessThanEqual
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    `LuauFloorDivide
  | `LuauModulo
  | `LuauMathFmod
  | `LuauMathAbs
  | `LuauMathFloor
  | `LuauMathCeil
  | `LuauMathSqrt
  | `LuauMathModf
  | `LuauBit32Band
  | `LuauBit32Bor
  | `LuauBit32Bxor
  | `LuauBit32Lshift
  | `LuauBit32Rshift
  | `LuauBit32Arshift
  | `LuauBit32Lrotate
  | `LuauBit32Rrotate
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    `LuauBit32Countlz
  | `LuauBit32Countrz
  | `LuauWrap
  | `LuauTableGet of Type.t * Type.t
  | `LuauTableSet of Type.t * Type.t
  | `LuauTableMove of Type.t
  | `LuauBufferReadU8
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    `LuauBufferReadI8
  | `LuauBufferReadU16
  | `LuauBufferReadI16
  | `LuauBufferReadU32
  | `LuauBufferReadF32
  | `LuauBufferReadF64
  | `LuauBufferWriteU8
  | `LuauBufferWriteU16
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    `LuauBufferWriteU32
  | `LuauBufferWriteF32
  | `LuauBufferWriteF64
  | `LuauArrayOf of Type.t * Type.t * string
  | `LuauLength of Type.t
  | `LuauBufferFromString of string
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    `LuauBufferLen
  | `LuauBufferFill
  | `LuauBufferCopy
  | `LuauBoxNew of Type.t
  | `LuauBoxGet of Type.t
  | `LuauBoxSet of Type.t
  | `LuauRetype of Type.t
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    `LuauArrayNew of Type.t * Type.t
  | `LuauBufferCreate
  | `LuauAllocated of Type.t
  | `LuauTry
  | `LuauCaught of Type.t list
  | `LuauOutOfBounds of Type.t ]

let name node =
  match node with
  | `LuauToNumber -> "Luau.ToNumber"
  | `LuauFromNumber _ -> "Luau.FromNumber"
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauBitsFromI64 -> "Luau.Bits.FromI64"
  | `LuauBitsLowFromI64 -> "Luau.Bits.LowFromI64"
  | `LuauBitsHighFromI64 -> "Luau.Bits.HighFromI64"
  | `LuauBitsIntoI64 -> "Luau.Bits.IntoI64"
  | `LuauBitsIntoF32 -> "Luau.Bits.IntoF32"
  | `LuauBitsFromF32 -> "Luau.Bits.FromF32"
  | `LuauBitsRoundF32 -> "Luau.Bits.RoundF32"
  | `LuauAdd -> "Luau.Add"
  | `LuauSubtract -> "Luau.Subtract"
  | `LuauMultiply -> "Luau.Multiply"
  | `LuauDivide -> "Luau.Divide"
  | `LuauNegate -> "Luau.Negate"
  | `LuauEqual -> "Luau.Equal"
  | `LuauLessThan -> "Luau.LessThan"
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauLessThanEqual -> "Luau.LessThanEqual"
  | `LuauFloorDivide -> "Luau.FloorDivide"
  | `LuauModulo -> "Luau.Modulo"
  | `LuauMathFmod -> "Luau.Math.Fmod"
  | `LuauMathAbs -> "Luau.Math.Abs"
  | `LuauMathFloor -> "Luau.Math.Floor"
  | `LuauMathCeil -> "Luau.Math.Ceil"
  | `LuauMathSqrt -> "Luau.Math.Sqrt"
  | `LuauMathModf -> "Luau.Math.Modf"
  | `LuauBit32Band -> "Luau.Bit32.Band"
  | `LuauBit32Bor -> "Luau.Bit32.Bor"
  | `LuauBit32Bxor -> "Luau.Bit32.Bxor"
  | `LuauBit32Lshift -> "Luau.Bit32.Lshift"
  | `LuauBit32Rshift -> "Luau.Bit32.Rshift"
  | `LuauBit32Arshift -> "Luau.Bit32.Arshift"
  | `LuauBit32Lrotate -> "Luau.Bit32.Lrotate"
  | `LuauBit32Rrotate -> "Luau.Bit32.Rrotate"
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauBit32Countlz -> "Luau.Bit32.Countlz"
  | `LuauBit32Countrz -> "Luau.Bit32.Countrz"
  | `LuauWrap -> "Luau.Wrap"
  | `LuauTableGet _ -> "Luau.Table.Get"
  | `LuauTableSet _ -> "Luau.Table.Set"
  | `LuauTableMove _ -> "Luau.Table.Move"
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauBufferReadU8 -> "Luau.Buffer.ReadU8"
  | `LuauBufferReadI8 -> "Luau.Buffer.ReadI8"
  | `LuauBufferReadU16 -> "Luau.Buffer.ReadU16"
  | `LuauBufferReadI16 -> "Luau.Buffer.ReadI16"
  | `LuauBufferReadU32 -> "Luau.Buffer.ReadU32"
  | `LuauBufferReadF32 -> "Luau.Buffer.ReadF32"
  | `LuauBufferReadF64 -> "Luau.Buffer.ReadF64"
  | `LuauBufferWriteU8 -> "Luau.Buffer.WriteU8"
  | `LuauBufferWriteU16 -> "Luau.Buffer.WriteU16"
  | `LuauBufferWriteU32 -> "Luau.Buffer.WriteU32"
  | `LuauBufferWriteF32 -> "Luau.Buffer.WriteF32"
  | `LuauBufferWriteF64 -> "Luau.Buffer.WriteF64"
  | `LuauArrayOf _ -> "Luau.ArrayOf"
  | `LuauLength _ -> "Luau.Length"
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauBufferFromString _ -> "Luau.Buffer.FromString"
  | `LuauBufferLen -> "Luau.Buffer.Len"
  | `LuauBufferFill -> "Luau.Buffer.Fill"
  | `LuauBufferCopy -> "Luau.Buffer.Copy"
  | `LuauBoxNew _ -> "Luau.Box.New"
  | `LuauBoxGet _ -> "Luau.Box.Get"
  | `LuauBoxSet _ -> "Luau.Box.Set"
  | `LuauRetype _ -> "Luau.Retype"
  | `LuauArrayNew _ -> "Luau.ArrayNew"
  | `LuauBufferCreate -> "Luau.BufferCreate"
  | `LuauAllocated _ -> "Luau.Allocated"
  | `LuauTry -> "Luau.Try"
  | `LuauCaught _ -> "Luau.Caught"
  | `LuauOutOfBounds _ -> "Luau.OutOfBounds"

let try_outputs types node =
  Type.boolean :: thrown types :: snd (TypeArena.signature types (Node.operand_type types node))

let output_count types node extended =
  match extended with
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauToNumber | `LuauFromNumber _ | `LuauBitsLowFromI64 | `LuauBitsHighFromI64 | `LuauBitsIntoI64 | `LuauBitsIntoF32
  | `LuauBitsFromF32 | `LuauBitsRoundF32 | `LuauAdd | `LuauSubtract | `LuauMultiply | `LuauDivide | `LuauNegate
  | `LuauEqual | `LuauLessThan | `LuauLessThanEqual | `LuauFloorDivide | `LuauModulo | `LuauMathFmod | `LuauMathAbs
  | `LuauMathFloor | `LuauMathCeil | `LuauMathSqrt | `LuauMathModf | `LuauBit32Band | `LuauBit32Bor | `LuauBit32Bxor
  | `LuauBit32Lshift | `LuauBit32Rshift | `LuauBit32Arshift | `LuauBit32Lrotate | `LuauBit32Rrotate | `LuauBit32Countlz
  | `LuauBit32Countrz | `LuauWrap | `LuauTableGet _ | `LuauTableSet _ | `LuauTableMove _ | `LuauBufferReadU8
  | `LuauBufferReadI8 | `LuauBufferReadU16 | `LuauBufferReadI16 | `LuauBufferReadU32 | `LuauBufferReadF32
  | `LuauBufferReadF64 | `LuauBufferWriteU8 | `LuauBufferWriteU16 | `LuauBufferWriteU32 | `LuauBufferWriteF32
  | `LuauBufferWriteF64 | `LuauArrayOf _ | `LuauLength _ | `LuauBufferFromString _ | `LuauBufferLen | `LuauBufferFill
  | `LuauBufferCopy | `LuauBoxNew _ | `LuauBoxGet _ | `LuauBoxSet _ | `LuauRetype _ | `LuauArrayNew _
  | `LuauBufferCreate | `LuauAllocated _ | `LuauOutOfBounds _ ->
      1
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauBitsFromI64 -> 2
  | `LuauTry -> List.length (try_outputs types node)
  | `LuauCaught payload_types -> List.length payload_types

let output_type types node extended port =
  match extended with
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauToNumber | `LuauBitsIntoF32 | `LuauBitsRoundF32 | `LuauAdd | `LuauSubtract | `LuauMultiply | `LuauDivide
  | `LuauNegate | `LuauFloorDivide | `LuauModulo | `LuauMathFmod | `LuauMathAbs | `LuauMathFloor | `LuauMathCeil
  | `LuauMathSqrt | `LuauMathModf | `LuauBufferReadU8 | `LuauBufferReadI8 | `LuauBufferReadU16 | `LuauBufferReadI16
  | `LuauBufferReadU32 | `LuauBufferReadF32 | `LuauBufferReadF64 ->
      Type.f64
  | `LuauFromNumber result -> Type.of_scalar result
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauBitsFromI64 | `LuauBitsLowFromI64 | `LuauBitsHighFromI64 | `LuauBit32Band | `LuauBit32Bor | `LuauBit32Bxor
  | `LuauBit32Lshift | `LuauBit32Rshift | `LuauBit32Arshift | `LuauBit32Lrotate | `LuauBit32Rrotate | `LuauBit32Countlz
  | `LuauBit32Countrz | `LuauWrap | `LuauLength _ | `LuauBufferLen ->
      Type.i32
  | `LuauBitsIntoI64 -> Type.i64
  | `LuauBitsFromF32 -> Type.f32
  | `LuauEqual | `LuauLessThan | `LuauLessThanEqual | `LuauAllocated _ -> Type.boolean
  | `LuauTableGet (element_type, _) | `LuauOutOfBounds element_type -> element_type
  | `LuauTableSet (array_type, _) | `LuauTableMove array_type | `LuauArrayNew (array_type, _) -> array_type
  | `LuauBufferWriteU8 | `LuauBufferWriteU16 | `LuauBufferWriteU32 | `LuauBufferWriteF32 | `LuauBufferWriteF64
  | `LuauBufferFromString _ | `LuauBufferFill | `LuauBufferCopy | `LuauBufferCreate ->
      Type.bytes
  | `LuauArrayOf (array_type, _, _) | `LuauBoxNew array_type | `LuauBoxSet array_type -> array_type
  | `LuauBoxGet box_type -> TypeArena.contents types box_type
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauRetype target -> target
  | `LuauTry -> List.nth (try_outputs types node) port
  | `LuauCaught payload_types -> List.nth payload_types port

let input_type types node extended index =
  match extended with
  | `LuauToNumber -> Node.admitted types node (fun operand -> Option.is_some (Type.scalar operand)) Type.f64
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauFromNumber _ | `LuauBitsFromF32 | `LuauBitsRoundF32 | `LuauAdd | `LuauSubtract | `LuauMultiply | `LuauDivide
  | `LuauNegate | `LuauLessThan | `LuauLessThanEqual | `LuauFloorDivide | `LuauModulo | `LuauMathFmod | `LuauMathAbs
  | `LuauMathFloor | `LuauMathCeil | `LuauMathSqrt | `LuauMathModf | `LuauBit32Band | `LuauBit32Bor | `LuauBit32Bxor
  | `LuauBit32Lshift | `LuauBit32Rshift | `LuauBit32Arshift | `LuauBit32Lrotate | `LuauBit32Rrotate | `LuauBit32Countlz
  | `LuauBit32Countrz | `LuauWrap ->
      Type.f64
  | `LuauBitsFromI64 | `LuauBitsLowFromI64 | `LuauBitsHighFromI64 -> Type.i64
  | `LuauBitsIntoI64 -> Type.i32
  | `LuauBitsIntoF32 -> Type.f32
  | `LuauEqual -> Node.operand_type types node
  | `LuauTableGet (_, array_type) -> if index = 0 then array_type else Type.f64
  | `LuauTableSet (array_type, value_type) -> ( match index with 0 -> array_type | 1 -> Type.f64 | _ -> value_type)
  | `LuauTableMove array_type -> ( match index with 0 | 2 -> array_type | _ -> Type.f64)
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauBufferReadU8 | `LuauBufferReadI8 | `LuauBufferReadU16 | `LuauBufferReadI16 | `LuauBufferReadU32
  | `LuauBufferReadF32 | `LuauBufferReadF64 | `LuauBufferWriteU8 | `LuauBufferWriteU16 | `LuauBufferWriteU32
  | `LuauBufferWriteF32 | `LuauBufferWriteF64 ->
      if index = 0 then Type.bytes else Type.f64
  | `LuauArrayOf _ | `LuauBufferFromString _ | `LuauOutOfBounds _ ->
      (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
      invalid_arg "Luau.OutOfBounds, ArrayOf and Buffer.FromString read nothing."
  | `LuauLength array_type | `LuauAllocated array_type -> array_type
  | `LuauBufferLen -> Type.bytes
  | `LuauBufferFill -> ( match index with 0 -> Type.bytes | 2 -> Type.i8 | _ -> Type.i32)
  | `LuauBufferCopy -> ( match index with 0 | 2 -> Type.bytes | _ -> Type.i32)
  | `LuauBoxNew box_type -> TypeArena.contents types box_type
  | `LuauBoxGet box_type -> box_type
  | `LuauBoxSet box_type -> if index = 0 then box_type else TypeArena.contents types box_type
  | `LuauRetype _ -> Node.operand_type types node
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauArrayNew (_, initial_type) -> if index = 0 then Type.i32 else initial_type
  | `LuauBufferCreate -> Type.i32
  | `LuauTry ->
      let callee = Node.operand_type types node in
      List.nth (callee :: fst (TypeArena.signature types callee)) index
  | `LuauCaught _ -> thrown types

let base node : Definitions.base =
  match node with
  | `LuauToNumber | `LuauFromNumber _ -> ViewOperation
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauBitsFromI64 | `LuauTableGet _ | `LuauBufferReadU8 | `LuauBufferReadI8 | `LuauBufferReadU16 | `LuauBufferReadI16
  | `LuauBufferReadU32 | `LuauBufferReadF32 | `LuauBufferReadF64 | `LuauArrayOf _ | `LuauBufferFromString _
  | `LuauBoxNew _ | `LuauBoxGet _ | `LuauArrayNew _ | `LuauBufferCreate | `LuauAllocated _ | `LuauTry | `LuauCaught _
  | `LuauOutOfBounds _ ->
      Operation
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauBitsLowFromI64 | `LuauBitsHighFromI64 | `LuauBitsIntoF32 | `LuauBitsFromF32 | `LuauBitsRoundF32 | `LuauNegate
  | `LuauMathAbs | `LuauMathFloor | `LuauMathCeil | `LuauMathSqrt | `LuauMathModf | `LuauBit32Countlz
  | `LuauBit32Countrz | `LuauWrap | `LuauLength _ | `LuauBufferLen | `LuauRetype _ ->
      UnaryOperation
  | `LuauBitsIntoI64 | `LuauAdd | `LuauSubtract | `LuauMultiply | `LuauDivide | `LuauEqual | `LuauLessThan
  | `LuauLessThanEqual | `LuauFloorDivide | `LuauModulo | `LuauMathFmod | `LuauBit32Band | `LuauBit32Bor
  | `LuauBit32Bxor | `LuauBit32Lshift | `LuauBit32Rshift | `LuauBit32Arshift | `LuauBit32Lrotate | `LuauBit32Rrotate ->
      BinaryOperation
  | `LuauTableSet _ | `LuauTableMove _ | `LuauBufferWriteU8 | `LuauBufferWriteU16 | `LuauBufferWriteU32
  | `LuauBufferWriteF32 | `LuauBufferWriteF64 | `LuauBufferFill | `LuauBufferCopy | `LuauBoxSet _ ->
      Write

let holds node =
  match node with
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauToNumber | `LuauFromNumber _ | `LuauBitsFromI64 | `LuauBitsLowFromI64 | `LuauBitsHighFromI64 | `LuauBitsIntoI64
  | `LuauBitsIntoF32 | `LuauBitsFromF32 | `LuauBitsRoundF32 | `LuauAdd | `LuauSubtract | `LuauMultiply | `LuauDivide
  | `LuauNegate | `LuauFloorDivide | `LuauModulo | `LuauMathFmod | `LuauMathAbs | `LuauMathFloor | `LuauMathCeil
  | `LuauMathSqrt | `LuauMathModf | `LuauBit32Band | `LuauBit32Bor | `LuauBit32Bxor | `LuauBit32Lshift
  | `LuauBit32Rshift | `LuauBit32Arshift | `LuauBit32Lrotate | `LuauBit32Rrotate | `LuauBit32Countlz | `LuauBit32Countrz
  | `LuauWrap | `LuauTableGet _ | `LuauTableSet _ | `LuauTableMove _ | `LuauBufferReadU8 | `LuauBufferReadI8
  | `LuauBufferReadU16 | `LuauBufferReadI16 | `LuauBufferReadU32 | `LuauBufferReadF32 | `LuauBufferReadF64
  | `LuauBufferWriteU8 | `LuauBufferWriteU16 | `LuauBufferWriteU32 | `LuauBufferWriteF32 | `LuauBufferWriteF64
  | `LuauArrayOf _ | `LuauLength _ | `LuauBufferFromString _ | `LuauBufferLen | `LuauBufferFill | `LuauBufferCopy
  | `LuauBoxNew _ | `LuauBoxGet _ | `LuauBoxSet _ | `LuauRetype _ | `LuauArrayNew _ | `LuauBufferCreate
  | `LuauAllocated _ | `LuauTry | `LuauCaught _ | `LuauOutOfBounds _ ->
      0
  | `LuauEqual -> Outcomes.equal
  | `LuauLessThan -> Outcomes.less
  | `LuauLessThanEqual -> Outcomes.less lor Outcomes.equal

let commutative node =
  match node with
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauToNumber | `LuauFromNumber _ | `LuauBitsFromI64 | `LuauBitsLowFromI64 | `LuauBitsHighFromI64 | `LuauBitsIntoI64
  | `LuauBitsIntoF32 | `LuauBitsFromF32 | `LuauBitsRoundF32 | `LuauSubtract | `LuauDivide | `LuauNegate | `LuauLessThan
  | `LuauLessThanEqual | `LuauFloorDivide | `LuauModulo | `LuauMathFmod | `LuauMathAbs | `LuauMathFloor | `LuauMathCeil
  | `LuauMathSqrt | `LuauMathModf | `LuauBit32Lshift | `LuauBit32Rshift | `LuauBit32Arshift | `LuauBit32Lrotate
  | `LuauBit32Rrotate | `LuauBit32Countlz | `LuauBit32Countrz | `LuauWrap | `LuauTableGet _ | `LuauTableSet _
  | `LuauTableMove _ | `LuauBufferReadU8 | `LuauBufferReadI8 | `LuauBufferReadU16 | `LuauBufferReadI16
  | `LuauBufferReadU32 | `LuauBufferReadF32 | `LuauBufferReadF64 | `LuauBufferWriteU8 | `LuauBufferWriteU16
  | `LuauBufferWriteU32 | `LuauBufferWriteF32 | `LuauBufferWriteF64 | `LuauArrayOf _ | `LuauLength _
  | `LuauBufferFromString _ | `LuauBufferLen | `LuauBufferFill | `LuauBufferCopy | `LuauBoxNew _ | `LuauBoxGet _
  | `LuauBoxSet _ | `LuauRetype _ | `LuauArrayNew _ | `LuauBufferCreate | `LuauAllocated _ | `LuauTry | `LuauCaught _
  | `LuauOutOfBounds _ ->
      false
  | `LuauAdd | `LuauMultiply | `LuauEqual | `LuauBit32Band | `LuauBit32Bor | `LuauBit32Bxor -> true

let unrepeatable node =
  match node with
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauToNumber | `LuauFromNumber _ | `LuauBitsFromI64 | `LuauBitsLowFromI64 | `LuauBitsHighFromI64 | `LuauBitsIntoI64
  | `LuauBitsIntoF32 | `LuauBitsFromF32 | `LuauBitsRoundF32 | `LuauAdd | `LuauSubtract | `LuauMultiply | `LuauDivide
  | `LuauNegate | `LuauEqual | `LuauLessThan | `LuauLessThanEqual | `LuauFloorDivide | `LuauModulo | `LuauMathFmod
  | `LuauMathAbs | `LuauMathFloor | `LuauMathCeil | `LuauMathSqrt | `LuauMathModf | `LuauBit32Band | `LuauBit32Bor
  | `LuauBit32Bxor | `LuauBit32Lshift | `LuauBit32Rshift | `LuauBit32Arshift | `LuauBit32Lrotate | `LuauBit32Rrotate
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  | `LuauBit32Countlz | `LuauBit32Countrz | `LuauWrap | `LuauTableGet _ | `LuauTableSet _ | `LuauBufferReadU8
  | `LuauBufferReadI8 | `LuauBufferReadU16 | `LuauBufferReadI16 | `LuauBufferReadU32 | `LuauBufferReadF32
  | `LuauBufferReadF64 | `LuauBufferWriteU8 | `LuauBufferWriteU16 | `LuauBufferWriteU32 | `LuauBufferWriteF32
  | `LuauBufferWriteF64 | `LuauLength _ | `LuauBufferLen | `LuauBufferFill | `LuauBoxGet _ | `LuauBoxSet _
  | `LuauRetype _ | `LuauAllocated _ | `LuauCaught _ | `LuauOutOfBounds _ ->
      false
  | `LuauTableMove _ | `LuauArrayOf _ | `LuauBufferFromString _ | `LuauBufferCopy | `LuauBoxNew _ | `LuauArrayNew _
  | `LuauBufferCreate | `LuauTry ->
      true

let array_access _ = None

let table_get types array key =
  let array_type = Link.type_ types array in
  Node.create (Extension (`LuauTableGet (TypeArena.element types array_type, array_type))) [| array; key |]

let table_set types array key value =
  Node.create (Extension (`LuauTableSet (Link.type_ types array, Link.type_ types value))) [| array; key; value |]

(* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
let array_new types length initial =
  let initial_type = Link.type_ types initial in
  Node.create
    (Extension (`LuauArrayNew (TypeArena.intern types (Array initial_type), initial_type)))
    [| length; initial |]

let allocated types array = Node.create (Extension (`LuauAllocated (Link.type_ types array))) [| array |]
let try_ callee arguments = Node.create (Extension `LuauTry) (Array.append [| callee |] arguments)
let caught thrown payload_types = Node.create (Extension (`LuauCaught payload_types)) [| thrown |]
