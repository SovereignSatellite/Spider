open IR

(** The core's integers are Rust's unsigned ones. *)
type TypeDefinition.extension += Signed of int | Usize | Outcome of Type.t list

val signed : 'x TypeArena.t -> int -> Type.t
val usize : 'x TypeArena.t -> Type.t
val describe : TypeDefinition.extension -> string

type t =
  [ (* Rust's [as], which saturates a float to an integer, NaN giving 0. *)
    `RustCast of Type.t
  | (* Of numbers, or of integers, undefined on an integer zero divisor or a signed overflow. *)
    `RustDivide
  | (* Of signed integers; undefined on a zero divisor. *)
    `RustWrappingRem
  | `RustWrappingShl of Type.t
  | (* Of an unsigned integer, filling with zeros. *)
    `RustWrappingShr of Type.t
  | (* Of a signed integer, filling with its sign. *)
    `RustSignedWrappingShr of Type.t
  | `RustRotateLeft of Type.t
  | `RustRotateRight of Type.t
  | `RustLessThan
  | `RustLessThanEqual
  | `RustEqual
  | `RustWrappingAdd
  | `RustWrappingSub
  | `RustWrappingMul
  | `RustBitAnd
  | `RustBitOr
  | `RustBitXor
  | (* Of unsigned integers; undefined on a zero divisor. *)
    `RustRemainder
  | `RustAdd
  | `RustSubtract
  | `RustMultiply
  | `RustNegate
  | `RustCopysign
  | `RustAbs
  | `RustSqrt
  | `RustCeil
  | `RustFloor
  | `RustTrunc
  | `RustRoundTiesEven
  | (* [from_bits] into the number type, and [to_bits] into the integer type. *)
    `RustFromBits of Type.t
  | `RustToBits of Type.t
  | (* [core::ptr::eq] of arrays or boxes. *)
    `RustPointerEqual
  | `RustLeadingZeros
  | `RustTrailingZeros
  | `RustCountOnes
  | (* An access out of bounds is undefined. Holds the element, array and index types. *)
    `RustElementGet of Type.t * Type.t * Type.t
  | (* Holds the array, index and value types. *)
    `RustElementSet of Type.t * Type.t * Type.t
  | (* Holds the array, index and value types. *)
    `RustElementFill of Type.t * Type.t * Type.t
  | (* Copies as if through a temporary. Holds the array and index types. *)
    `RustElementCopy of Type.t * Type.t
  | (* Holds the array type. *)
    `RustLength of Type.t
  | (* Reads the value's bytes at the byte offset with [read_unaligned]. Holds the memory, offset and value types. *)
    `RustRead of Type.t * Type.t * Type.t
  | (* Writes the value's bytes at the byte offset with [write_unaligned]. Holds the memory, offset and value types. *)
    `RustWrite of Type.t * Type.t * Type.t
  | (* The runtime's array of the constant elements; holds the array and element types and the elements' bytes. *)
    `RustArrayOf of Type.t * Type.t * string
  | (* A box is the runtime's pointer to a struct of the contents' leaves; holds the box type. *)
    `RustBoxNew of Type.t
  | `RustBoxGet of Type.t
  | `RustBoxSet of Type.t
  | (* [transmute] of the value to the type. *)
    `RustRetype of Type.t
  | (* A leaf the runtime could not allocate is null. Holds the array and initial types. *)
    `RustArrayNew of Type.t * Type.t
  | `RustAllocated of Type.t
  | (* Applies the callee, giving its outcome: its results, or what it threw. *)
    `RustTry
  | `RustThrew
  | (* The results of an outcome that did not throw; undefined on one that did. *)
    `RustReturned
  | (* The payload of an outcome that threw, freeing the outcome; undefined on one that did not throw or without the
       payload types. *)
    `RustCaught of Type.t list
  | (* Panics, which no [RustTry] catches; its result is never produced. *)
    `RustOutOfBounds of Type.t ]

val name : t -> string
val output_count : 'x TypeArena.t -> 'x Node.t -> t -> int
val output_type : 'x TypeArena.t -> 'x Node.t -> t -> int -> Type.t
val input_type : 'x TypeArena.t -> 'x Node.t -> t -> int -> Type.t
val base : t -> Definitions.base
val holds : t -> int
val commutative : t -> bool
val unrepeatable : t -> bool
val array_access : t -> Definitions.array_access option

val element_get :
  ([> `RustElementGet of Type.t * Type.t * Type.t ] as 'a) TypeArena.t -> 'a Link.t -> 'a Link.t -> 'a Node.t

val element_set :
  ([> `RustElementSet of Type.t * Type.t * Type.t ] as 'a) TypeArena.t ->
  'a Link.t ->
  'a Link.t ->
  'a Link.t ->
  'a Node.t

val element_fill :
  ([> `RustElementFill of Type.t * Type.t * Type.t ] as 'a) TypeArena.t ->
  'a Link.t ->
  'a Link.t ->
  'a Link.t ->
  'a Link.t ->
  'a Node.t

val element_copy :
  ([> `RustElementCopy of Type.t * Type.t ] as 'a) TypeArena.t ->
  'a Link.t ->
  'a Link.t ->
  'a Link.t ->
  'a Link.t ->
  'a Link.t ->
  'a Node.t

val length : ([> `RustLength of Type.t ] as 'a) TypeArena.t -> 'a Link.t -> 'a Node.t

val read :
  ([> `RustRead of Type.t * Type.t * Type.t ] as 'a) TypeArena.t -> 'a Link.t -> 'a Link.t -> Type.t -> 'a Node.t

val write :
  ([> `RustWrite of Type.t * Type.t * Type.t ] as 'a) TypeArena.t -> 'a Link.t -> 'a Link.t -> 'a Link.t -> 'a Node.t

val array_new : ([> `RustArrayNew of Type.t * Type.t ] as 'a) TypeArena.t -> 'a Link.t -> 'a Link.t -> 'a Node.t
val allocated : ([> `RustAllocated of Type.t ] as 'a) TypeArena.t -> 'a Link.t -> 'a Node.t
val try_ : ([> `RustTry ] as 'a) Link.t -> 'a Link.t array -> 'a Node.t
val threw : ([> `RustThrew ] as 'a) Link.t -> 'a Node.t
val returned : ([> `RustReturned ] as 'a) Link.t -> 'a Node.t
val caught : ([> `RustCaught of Type.t list ] as 'a) Link.t -> Type.t list -> 'a Node.t
