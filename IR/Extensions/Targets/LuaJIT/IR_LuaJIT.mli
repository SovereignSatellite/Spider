open IR

(* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)

(** What a throw raised: its payload's one leaf, or a table of its leaves. FFI's [uint64_t] holds an I64 for the
    operations that read it as unsigned. *)
type TypeDefinition.extension += Thrown | Unsigned

val unsigned : 'x TypeArena.t -> Type.t
val describe : TypeDefinition.extension -> string

type t =
  (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
  [ (* The number LuaJIT holds for the value: an integer of 8 or 16 bits as its unsigned value, an I32 as its signed
       value, an F32 as its bits' signed value, and an F64 as itself. An I64 is FFI's [int64_t], not a number. *)
    `LuaJITToNumber
  | (* The inverse of [LuaJITToNumber]; undefined unless the result type holds the number. *)
    `LuaJITFromNumber of Scalar.t
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    (* The F64 the I64's bits hold, through the scratch union; undefined where they are a NaN other than LuaJIT's of
       either sign, as LuaJIT reads other NaNs as other values. *)
    `LuaJITBitsIntoF64
  | `LuaJITBitsFromF64
  | (* The F32's numeric value, through the scratch union; undefined where it is a NaN other than LuaJIT's of either
       sign. *)
    `LuaJITBitsIntoF32
  | (* The number rounded to the nearest F32, ties to even. *)
    `LuaJITBitsFromF32
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    (* Lua's [+], [-], [*] and [/] of two numbers, or of two [int64_t]s or [Unsigned]s modulo 2^64, where [/] truncates
       toward zero and is undefined on a zero divisor or where the quotient overflows. *)
    `LuaJITAdd
  | `LuaJITSubtract
  | `LuaJITMultiply
  | `LuaJITDivide
  | `LuaJITNegate
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    (* Lua's [==] of two values of one type: numbers and 64-bit integers by value, tables and cdata by identity. *)
    `LuaJITEqual
  | (* Lua's [<] of two numbers, [int64_t]s or [Unsigned]s, and [<=] of two numbers. *)
    `LuaJITLessThan
  | `LuaJITLessThanEqual
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    (* Lua's [%]: of numbers, the left less the right times the floor of their quotient; of [int64_t]s or [Unsigned]s,
       C's remainder, undefined on a zero divisor or where the quotient overflows. *)
    `LuaJITModulo
  | `LuaJITMathFmod
  | (* [math.abs], [math.floor], [math.ceil] and [math.sqrt], and [math.modf]'s integral part. *)
    `LuaJITMathAbs
  | `LuaJITMathFloor
  | `LuaJITMathCeil
  | `LuaJITMathSqrt
  | `LuaJITMathModf
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    (* A [bit] node's operands are numbers, giving an I32, or I64s, giving an I64; a number other than an integer of
       magnitude below 2^51 is undefined, and [bit] takes it modulo 2^32. *)
    `LuaJITBitBand
  | `LuaJITBitBor
  | `LuaJITBitBxor
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    (* Shifts by the amount modulo the width. *)
    `LuaJITBitLshift
  | (* Fills with zeros; shifts by the amount modulo the width. *)
    `LuaJITBitRshift
  | (* Shifts by the amount modulo the width. *)
    `LuaJITBitArshift
  | (* Rotates by the amount modulo the width. *)
    `LuaJITBitRol
  | (* Rotates by the amount modulo the width. *)
    `LuaJITBitRor
  | (* A number modulo 2^32, as a [bit] operand, or an I64's low 32 bits. *)
    `LuaJITBitTobit
  | (* [ffi.cast] to an I64 or [Unsigned]: of a number, truncated toward zero, undefined unless from −2^63 below 2^63
       for an I64 or below 2^64 for [Unsigned]; of an I64 or [Unsigned], its bits. *)
    `LuaJITCast of Type.t
  | (* [tonumber] of a 64-bit integer, rounded to nearest, ties to even. *)
    `LuaJITTonumber
  | (* The array is a table keyed from 1; an access out of bounds is undefined. *)
    `LuaJITTableGet of Type.t * Type.t
  | `LuaJITTableSet of Type.t * Type.t
  | (* [table.move] of the source's keys from the first to the last into the destination from the key, holding the
       table's type; its inputs are the destination, the key, the source, the first and the last. *)
    `LuaJITTableMove of Type.t
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    (* Through a pointer of the type at the address plus the offset into the bytes; an access out of bounds is
       undefined. *)
    `LuaJITReadU8
  | `LuaJITReadU16
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    `LuaJITReadI32
  | `LuaJITReadI64
  | (* Undefined where the bytes are a NaN. *)
    `LuaJITReadF64
  | (* An integer write stores the integer modulo 2^width. *)
    `LuaJITWriteU8
  | `LuaJITWriteU16
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    `LuaJITWriteI32
  | `LuaJITWriteI64
  | `LuaJITWriteF64
  | (* A table of the constant elements, with its length; holds the array and element types and the elements' bytes. *)
    `LuaJITArrayOf of Type.t * Type.t * string
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    (* A table's or [Bytes]' [length] field. *)
    `LuaJITLength of Type.t
  | (* [Bytes] of the string's bytes. *)
    `LuaJITBytesFromString of string
  | (* [Array.Fill] and [Array.Copy] of [Bytes]. *)
    `LuaJITBytesFill
  | `LuaJITBytesCopy
  | (* A box is a table of the contents' leaves; holds the box type. *)
    `LuaJITBoxNew of Type.t
  | `LuaJITBoxGet of Type.t
  | `LuaJITBoxSet of Type.t
  | (* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
    (* The value as the type, which LuaJIT does not check. *)
    `LuaJITRetype of Type.t
  | (* A table of the length, or [nil] where the runtime could not allocate it. *)
    `LuaJITArrayNew of Type.t
  | (* [Bytes] of the length, each zero, or [nil] where the runtime could not allocate them. *)
    `LuaJITBytesNew
  | `LuaJITAllocated of Type.t
  | (* Applies the callee, giving whether it threw, what it threw, then its results, each [nil] where it threw. *)
    `LuaJITTry
  | (* The payload of what a [LuaJITTry] gave as thrown; undefined unless it has the payload types. *)
    `LuaJITCaught of Type.t list
  | (* Raises an error, which no [LuaJITTry] catches; its result is never produced. *)
    `LuaJITOutOfBounds of Type.t ]

val name : t -> string
val output_count : 'x TypeArena.t -> 'x Node.t -> t -> int
val output_type : 'x TypeArena.t -> 'x Node.t -> t -> int -> Type.t
val input_type : 'x TypeArena.t -> 'x Node.t -> t -> int -> Type.t
val base : t -> Definitions.base
val holds : t -> int
val commutative : t -> bool
val unrepeatable : t -> bool
val array_access : t -> Definitions.array_access option
val table_get : ([> `LuaJITTableGet of Type.t * Type.t ] as 'a) TypeArena.t -> 'a Link.t -> 'a Link.t -> 'a Node.t

val table_set :
  ([> `LuaJITTableSet of Type.t * Type.t ] as 'a) TypeArena.t -> 'a Link.t -> 'a Link.t -> 'a Link.t -> 'a Node.t

(* LuaJIT: an operation of the Lua language, its libraries or its I64s as FFI longs. *)
val array_new : Type.t -> ([> `LuaJITArrayNew of Type.t ] as 'a) Link.t -> 'a Node.t
val allocated : ([> `LuaJITAllocated of Type.t ] as 'a) TypeArena.t -> 'a Link.t -> 'a Node.t
val try_ : ([> `LuaJITTry ] as 'a) Link.t -> 'a Link.t array -> 'a Node.t
val caught : ([> `LuaJITCaught of Type.t list ] as 'a) Link.t -> Type.t list -> 'a Node.t
