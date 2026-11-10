open IR

(** What a throw raised: its payload's one leaf, or a table of its leaves. *)
type TypeDefinition.extension += Thrown

val describe : TypeDefinition.extension -> string

type t =
  (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
  [ (* The number Luau holds for the value: an integer of up to 32 bits as its unsigned value, an F32 as its bits'
       unsigned value, an I64 as the F64 holding its bits, and an F64 as itself. *)
    `LuauToNumber
  | (* The inverse of [LuauToNumber]; undefined unless the result type holds the number. *)
    `LuauFromNumber of Scalar.t
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    `LuauBitsFromI64
  | `LuauBitsLowFromI64
  | `LuauBitsHighFromI64
  | `LuauBitsIntoI64
  | (* The F32's numeric value, not the number of its bits that [LuauToNumber] gives. *)
    `LuauBitsIntoF32
  | (* The number rounded to the nearest F32, ties to even. *)
    `LuauBitsFromF32
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    (* The number rounded to the nearest F32, as a number. *)
    `LuauBitsRoundF32
  | `LuauAdd
  | `LuauSubtract
  | `LuauMultiply
  | `LuauDivide
  | `LuauNegate
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    (* Luau's [==] of two values of one type: numbers by value, tables by identity. *)
    `LuauEqual
  | `LuauLessThan
  | `LuauLessThanEqual
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    (* Luau's [//], computed as [math.floor(left / right)]. *)
    `LuauFloorDivide
  | (* Luau's [%], computed as [left - math.floor(left / right) * right]. *)
    `LuauModulo
  | `LuauMathFmod
  | (* [math.abs], [math.floor], [math.ceil] and [math.sqrt], and [math.modf]'s integral part. *)
    `LuauMathAbs
  | `LuauMathFloor
  | `LuauMathCeil
  | `LuauMathSqrt
  | `LuauMathModf
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    (* A [bit32] node takes an operand truncated toward zero, modulo 2^32; one from 2^63 in magnitude is undefined. *)
    `LuauBit32Band
  | `LuauBit32Bor
  | `LuauBit32Bxor
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    (* A shift by more than 32 is undefined; by 32, the result is zero. *)
    `LuauBit32Lshift
  | (* Fills with zeros; a shift by more than 32 is undefined, and by 32 the result is zero. *)
    `LuauBit32Rshift
  | (* A shift by more than 32 is undefined; by 32, every bit is the sign bit. *)
    `LuauBit32Arshift
  | (* A rotation by more than 31 is undefined. *)
    `LuauBit32Lrotate
  | (* A rotation by more than 31 is undefined. *)
    `LuauBit32Rrotate
  | `LuauBit32Countlz
  | `LuauBit32Countrz
  | (* The number as a [bit32] node takes an operand; Luau has no function doing only this, so it prints as
       [bit32.bor] of the number and zero. *)
    `LuauWrap
  | (* The array is a table keyed from 1; an access out of bounds is undefined. *)
    `LuauTableGet of Type.t * Type.t
  | `LuauTableSet of Type.t * Type.t
  | (* [table.move] of the source's keys from the first to the last into the destination from the key, holding the
       table's type; its inputs are the destination, the key, the source, the first and the last. *)
    `LuauTableMove of Type.t
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    (* An offset out of bounds is undefined. *)
    `LuauBufferReadU8
  | `LuauBufferReadI8
  | `LuauBufferReadU16
  | `LuauBufferReadI16
  | `LuauBufferReadU32
  | `LuauBufferReadF32
  | `LuauBufferReadF64
  | (* An integer write stores the number as a [bit32] node takes an operand, modulo 2^width; an offset out of bounds
       is undefined. *)
    `LuauBufferWriteU8
  | `LuauBufferWriteU16
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    `LuauBufferWriteU32
  | `LuauBufferWriteF32
  | `LuauBufferWriteF64
  | (* A table of the constant elements, with its length; holds the array and element types and the elements' bytes. *)
    `LuauArrayOf of Type.t * Type.t * string
  | (* A table's [length] field. *)
    `LuauLength of Type.t
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    (* [buffer.fromstring], [buffer.len], [buffer.fill] and [buffer.copy], their inputs in the library's order. *)
    `LuauBufferFromString of string
  | `LuauBufferLen
  | `LuauBufferFill
  | `LuauBufferCopy
  | (* A box is a table of the contents' leaves; holds the box type. *)
    `LuauBoxNew of Type.t
  | `LuauBoxGet of Type.t
  | `LuauBoxSet of Type.t
  | (* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
    (* The value as the type, erased through [any]. *)
    `LuauRetype of Type.t
  | (* [table.create] of the length and the initial element for each leaf, or [nil] where the runtime could not
       allocate it. *)
    `LuauArrayNew of Type.t * Type.t
  | (* [buffer.create] of the length, or [nil] where the runtime could not allocate it. *)
    `LuauBufferCreate
  | `LuauAllocated of Type.t
  | (* Applies the callee, giving whether it threw, what it threw, then its results, each [nil] where it threw. *)
    `LuauTry
  | (* The payload of what a [LuauTry] gave as thrown; undefined unless it has the payload types. *)
    `LuauCaught of Type.t list
  | (* Raises an error, which no [LuauTry] catches; its result is never produced. *)
    `LuauOutOfBounds of Type.t ]

val name : t -> string
val output_count : 'x TypeArena.t -> 'x Node.t -> t -> int
val output_type : 'x TypeArena.t -> 'x Node.t -> t -> int -> Type.t
val input_type : 'x TypeArena.t -> 'x Node.t -> t -> int -> Type.t
val base : t -> Definitions.base
val holds : t -> int
val commutative : t -> bool
val unrepeatable : t -> bool
val array_access : t -> Definitions.array_access option
val table_get : ([> `LuauTableGet of Type.t * Type.t ] as 'a) TypeArena.t -> 'a Link.t -> 'a Link.t -> 'a Node.t

val table_set :
  ([> `LuauTableSet of Type.t * Type.t ] as 'a) TypeArena.t -> 'a Link.t -> 'a Link.t -> 'a Link.t -> 'a Node.t

(* Luau: an operation of the Luau language, its libraries or its I64s as two words. *)
val array_new : ([> `LuauArrayNew of Type.t * Type.t ] as 'a) TypeArena.t -> 'a Link.t -> 'a Link.t -> 'a Node.t
val allocated : ([> `LuauAllocated of Type.t ] as 'a) TypeArena.t -> 'a Link.t -> 'a Node.t
val try_ : ([> `LuauTry ] as 'a) Link.t -> 'a Link.t array -> 'a Node.t
val caught : ([> `LuauCaught of Type.t list ] as 'a) Link.t -> Type.t list -> 'a Node.t
