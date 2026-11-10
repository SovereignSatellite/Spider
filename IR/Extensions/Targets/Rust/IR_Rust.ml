open IR

type TypeDefinition.extension += Signed of int | Usize | Outcome of Type.t list

let signed types bits = TypeArena.extension types (Signed bits)
let usize types = TypeArena.extension types Usize
let outcome types results = TypeArena.extension types (Outcome results)

let describe definition =
  match definition with
  | Signed bits -> "Signed " ^ string_of_int bits
  | Usize -> "Usize"
  | Outcome results ->
      "Outcome ["
      ^ String.concat "; " (List.map (fun (result : Type.t) -> "Type " ^ string_of_int (result :> int)) results)
      ^ "]"
  | _ -> invalid_arg "Rust defines no such type."

let results types outcome =
  match TypeArena.definition types outcome with
  | Extension (Outcome results) -> results
  | _ -> invalid_arg ("Expected an outcome but found " ^ TypeArena.describe types outcome ^ ".")

type t =
  [ `RustCast of Type.t
  | `RustDivide
  | `RustWrappingRem
  | `RustWrappingShl of Type.t
  | `RustWrappingShr of Type.t
  | `RustSignedWrappingShr of Type.t
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
  | `RustRemainder
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
  | `RustFromBits of Type.t
  | `RustToBits of Type.t
  | `RustPointerEqual
  | `RustLeadingZeros
  | `RustTrailingZeros
  | `RustCountOnes
  | `RustElementGet of Type.t * Type.t * Type.t
  | `RustElementSet of Type.t * Type.t * Type.t
  | `RustElementFill of Type.t * Type.t * Type.t
  | `RustElementCopy of Type.t * Type.t
  | `RustLength of Type.t
  | `RustRead of Type.t * Type.t * Type.t
  | `RustWrite of Type.t * Type.t * Type.t
  | `RustArrayOf of Type.t * Type.t * string
  | `RustBoxNew of Type.t
  | `RustBoxGet of Type.t
  | `RustBoxSet of Type.t
  | `RustRetype of Type.t
  | `RustArrayNew of Type.t * Type.t
  | `RustAllocated of Type.t
  | `RustTry
  | `RustThrew
  | `RustReturned
  | `RustCaught of Type.t list
  | `RustOutOfBounds of Type.t ]

let name node =
  match node with
  | `RustCast _ -> "Rust.Cast"
  | `RustDivide -> "Rust.Divide"
  | `RustWrappingRem -> "Rust.WrappingRem"
  | `RustWrappingShl _ -> "Rust.WrappingShl"
  | `RustWrappingShr _ -> "Rust.WrappingShr"
  | `RustSignedWrappingShr _ -> "Rust.SignedWrappingShr"
  | `RustRotateLeft _ -> "Rust.RotateLeft"
  | `RustRotateRight _ -> "Rust.RotateRight"
  | `RustLessThan -> "Rust.LessThan"
  | `RustLessThanEqual -> "Rust.LessThanEqual"
  | `RustEqual -> "Rust.Equal"
  | `RustWrappingAdd -> "Rust.WrappingAdd"
  | `RustWrappingSub -> "Rust.WrappingSub"
  | `RustWrappingMul -> "Rust.WrappingMul"
  | `RustBitAnd -> "Rust.BitAnd"
  | `RustBitOr -> "Rust.BitOr"
  | `RustBitXor -> "Rust.BitXor"
  | `RustRemainder -> "Rust.Remainder"
  | `RustAdd -> "Rust.Add"
  | `RustSubtract -> "Rust.Subtract"
  | `RustMultiply -> "Rust.Multiply"
  | `RustNegate -> "Rust.Negate"
  | `RustCopysign -> "Rust.Copysign"
  | `RustAbs -> "Rust.Abs"
  | `RustSqrt -> "Rust.Sqrt"
  | `RustCeil -> "Rust.Ceil"
  | `RustFloor -> "Rust.Floor"
  | `RustTrunc -> "Rust.Trunc"
  | `RustRoundTiesEven -> "Rust.RoundTiesEven"
  | `RustFromBits _ -> "Rust.FromBits"
  | `RustToBits _ -> "Rust.ToBits"
  | `RustPointerEqual -> "Rust.PointerEqual"
  | `RustLeadingZeros -> "Rust.LeadingZeros"
  | `RustTrailingZeros -> "Rust.TrailingZeros"
  | `RustCountOnes -> "Rust.CountOnes"
  | `RustElementGet _ -> "Rust.ElementGet"
  | `RustElementSet _ -> "Rust.ElementSet"
  | `RustElementFill _ -> "Rust.ElementFill"
  | `RustElementCopy _ -> "Rust.ElementCopy"
  | `RustLength _ -> "Rust.Length"
  | `RustRead _ -> "Rust.Read"
  | `RustWrite _ -> "Rust.Write"
  | `RustArrayOf _ -> "Rust.ArrayOf"
  | `RustBoxNew _ -> "Rust.Box.New"
  | `RustBoxGet _ -> "Rust.Box.Get"
  | `RustBoxSet _ -> "Rust.Box.Set"
  | `RustRetype _ -> "Rust.Retype"
  | `RustArrayNew _ -> "Rust.ArrayNew"
  | `RustAllocated _ -> "Rust.Allocated"
  | `RustTry -> "Rust.Try"
  | `RustThrew -> "Rust.Threw"
  | `RustReturned -> "Rust.Returned"
  | `RustCaught _ -> "Rust.Caught"
  | `RustOutOfBounds _ -> "Rust.OutOfBounds"

let returned_results types node = results types (Node.operand_type types node)

let output_count types node extended =
  match extended with
  | `RustReturned -> List.length (returned_results types node)
  | `RustCaught payload_types -> List.length payload_types
  | `RustCast _ | `RustDivide | `RustWrappingRem | `RustWrappingShl _ | `RustWrappingShr _ | `RustSignedWrappingShr _
  | `RustRotateLeft _ | `RustRotateRight _ | `RustLessThan | `RustLessThanEqual | `RustEqual | `RustWrappingAdd
  | `RustWrappingSub | `RustWrappingMul | `RustBitAnd | `RustBitOr | `RustBitXor | `RustRemainder | `RustAdd
  | `RustSubtract | `RustMultiply | `RustNegate | `RustCopysign | `RustAbs | `RustSqrt | `RustCeil | `RustFloor
  | `RustTrunc | `RustRoundTiesEven | `RustFromBits _ | `RustToBits _ | `RustPointerEqual | `RustLeadingZeros
  | `RustTrailingZeros | `RustCountOnes | `RustElementGet _ | `RustElementSet _ | `RustElementFill _
  | `RustElementCopy _ | `RustLength _ | `RustRead _ | `RustWrite _ | `RustArrayOf _ | `RustBoxNew _ | `RustBoxGet _
  | `RustBoxSet _ | `RustRetype _ | `RustArrayNew _ | `RustAllocated _ | `RustTry | `RustThrew | `RustOutOfBounds _ ->
      1

let integer types value_type =
  match TypeArena.definition types value_type with
  | Structural (Integer _) | Extension (Signed _ | Usize) -> true
  | Structural (Number _ | Sum _ | Function _ | Array _ | Opaque) | Box _ | Extension _ -> false

let number value_type = value_type = Type.f32 || value_type = Type.f64

let rec output_type types node extended port =
  match extended with
  | `RustElementGet (element_type, _, _) -> element_type
  | `RustDivide | `RustWrappingRem | `RustWrappingAdd | `RustWrappingSub | `RustWrappingMul | `RustBitAnd | `RustBitOr
  | `RustBitXor | `RustRemainder | `RustAdd | `RustSubtract | `RustMultiply | `RustNegate | `RustCopysign | `RustAbs
  | `RustSqrt | `RustCeil | `RustFloor | `RustTrunc | `RustRoundTiesEven ->
      input_type types node extended 0
  | `RustArrayOf (array_type, _, _) | `RustBoxNew array_type | `RustBoxSet array_type -> array_type
  | `RustBoxGet box_type -> TypeArena.contents types box_type
  | `RustFromBits target | `RustToBits target | `RustRetype target | `RustCast target -> target
  | `RustLength _ -> usize types
  | `RustElementSet (array_type, _, _)
  | `RustElementFill (array_type, _, _)
  | `RustElementCopy (array_type, _)
  | `RustWrite (array_type, _, _)
  | `RustArrayNew (array_type, _) ->
      array_type
  | `RustAllocated _ | `RustLessThan | `RustLessThanEqual | `RustEqual | `RustPointerEqual | `RustThrew -> Type.boolean
  | `RustWrappingShl value_type
  | `RustWrappingShr value_type
  | `RustSignedWrappingShr value_type
  | `RustRotateLeft value_type
  | `RustRotateRight value_type ->
      value_type
  | `RustLeadingZeros | `RustTrailingZeros | `RustCountOnes -> Type.i32
  | `RustTry -> outcome types (snd (TypeArena.signature types (Node.operand_type types node)))
  | `RustReturned -> List.nth (returned_results types node) port
  | `RustCaught payload_types -> List.nth payload_types port
  | `RustRead (_, _, value_type) | `RustOutOfBounds value_type -> value_type

and input_type types node extended index =
  match extended with
  | `RustElementGet (_, array_type, index_type) -> if index = 0 then array_type else index_type
  | `RustElementSet (array_type, index_type, value_type) | `RustWrite (array_type, index_type, value_type) -> (
      match index with 0 -> array_type | 1 -> index_type | _ -> value_type)
  | `RustElementFill (array_type, index_type, value_type) -> (
      match index with 0 -> array_type | 3 -> value_type | _ -> index_type)
  | `RustElementCopy (array_type, index_type) -> ( match index with 0 | 2 -> array_type | _ -> index_type)
  | `RustLength array_type | `RustAllocated array_type -> array_type
  | `RustArrayNew (_, initial_type) -> if index = 0 then Type.i32 else initial_type
  | `RustEqual | `RustPointerEqual | `RustRetype _ -> Node.operand_type types node
  | `RustCast _ | `RustDivide | `RustLessThan ->
      Node.admitted types node (fun operand -> integer types operand || number operand) Type.f64
  | `RustWrappingRem | `RustWrappingAdd | `RustWrappingSub | `RustWrappingMul | `RustBitAnd | `RustBitOr | `RustBitXor
  | `RustRemainder | `RustLeadingZeros | `RustTrailingZeros | `RustCountOnes ->
      Node.admitted types node (integer types) Type.i64
  | `RustAdd | `RustSubtract | `RustMultiply | `RustNegate | `RustCopysign | `RustAbs | `RustSqrt | `RustCeil
  | `RustFloor | `RustTrunc | `RustRoundTiesEven | `RustLessThanEqual ->
      Node.admitted types node number Type.f64
  | `RustFromBits target -> if target = Type.f32 then Type.i32 else Type.i64
  | `RustToBits target -> if target = Type.i32 then Type.f32 else Type.f64
  | `RustBoxNew box_type -> TypeArena.contents types box_type
  | `RustBoxGet box_type -> box_type
  | `RustBoxSet box_type -> if index = 0 then box_type else TypeArena.contents types box_type
  | `RustWrappingShl value_type
  | `RustWrappingShr value_type
  | `RustSignedWrappingShr value_type
  | `RustRotateLeft value_type
  | `RustRotateRight value_type ->
      if index = 0 then value_type else Type.i32
  | `RustTry ->
      let callee = Node.operand_type types node in
      List.nth (callee :: fst (TypeArena.signature types callee)) index
  | `RustThrew | `RustReturned | `RustCaught _ -> Node.operand_type types node
  | `RustRead (memory_type, offset_type, _) -> if index = 0 then memory_type else offset_type
  | `RustOutOfBounds _ | `RustArrayOf _ -> invalid_arg "Rust.OutOfBounds and ArrayOf read nothing."

let base node : Definitions.base =
  match node with
  | `RustElementSet _ | `RustElementFill _ | `RustElementCopy _ | `RustWrite _ | `RustBoxSet _ -> Write
  | `RustCast _ | `RustLeadingZeros | `RustTrailingZeros | `RustCountOnes | `RustNegate | `RustAbs | `RustSqrt
  | `RustCeil | `RustFloor | `RustTrunc | `RustRoundTiesEven | `RustFromBits _ | `RustToBits _ | `RustRetype _ ->
      UnaryOperation
  | `RustDivide | `RustWrappingRem | `RustLessThan | `RustLessThanEqual | `RustEqual | `RustWrappingAdd
  | `RustWrappingSub | `RustWrappingMul | `RustBitAnd | `RustBitOr | `RustBitXor | `RustRemainder | `RustAdd
  | `RustSubtract | `RustMultiply | `RustCopysign | `RustPointerEqual ->
      BinaryOperation
  | `RustArrayOf _ | `RustBoxNew _ | `RustBoxGet _ | `RustElementGet _ | `RustLength _ | `RustArrayNew _
  | `RustAllocated _ | `RustWrappingShl _ | `RustWrappingShr _ | `RustSignedWrappingShr _ | `RustRotateLeft _
  | `RustRotateRight _ | `RustTry | `RustThrew | `RustReturned | `RustCaught _ | `RustRead _ | `RustOutOfBounds _ ->
      Operation

let holds node =
  match node with
  | `RustEqual | `RustPointerEqual -> Outcomes.equal
  | `RustLessThan -> Outcomes.less
  | `RustLessThanEqual -> Outcomes.less lor Outcomes.equal
  | `RustCast _ | `RustDivide | `RustWrappingRem | `RustWrappingShl _ | `RustWrappingShr _ | `RustSignedWrappingShr _
  | `RustRotateLeft _ | `RustRotateRight _ | `RustWrappingAdd | `RustWrappingSub | `RustWrappingMul | `RustBitAnd
  | `RustBitOr | `RustBitXor | `RustRemainder | `RustAdd | `RustSubtract | `RustMultiply | `RustNegate | `RustCopysign
  | `RustAbs | `RustSqrt | `RustCeil | `RustFloor | `RustTrunc | `RustRoundTiesEven | `RustFromBits _ | `RustToBits _
  | `RustLeadingZeros | `RustTrailingZeros | `RustCountOnes | `RustElementGet _ | `RustElementSet _ | `RustElementFill _
  | `RustElementCopy _ | `RustLength _ | `RustRead _ | `RustWrite _ | `RustArrayOf _ | `RustBoxNew _ | `RustBoxGet _
  | `RustBoxSet _ | `RustRetype _ | `RustArrayNew _ | `RustAllocated _ | `RustTry | `RustThrew | `RustReturned
  | `RustCaught _ | `RustOutOfBounds _ ->
      0

let commutative node =
  match node with
  | `RustEqual | `RustWrappingAdd | `RustWrappingMul | `RustBitAnd | `RustBitOr | `RustBitXor | `RustAdd | `RustMultiply
  | `RustPointerEqual ->
      true
  | `RustCast _ | `RustDivide | `RustWrappingRem | `RustWrappingShl _ | `RustWrappingShr _ | `RustSignedWrappingShr _
  | `RustRotateLeft _ | `RustRotateRight _ | `RustLessThan | `RustLessThanEqual | `RustWrappingSub | `RustRemainder
  | `RustSubtract | `RustNegate | `RustCopysign | `RustAbs | `RustSqrt | `RustCeil | `RustFloor | `RustTrunc
  | `RustRoundTiesEven | `RustFromBits _ | `RustToBits _ | `RustLeadingZeros | `RustTrailingZeros | `RustCountOnes
  | `RustElementGet _ | `RustElementSet _ | `RustElementFill _ | `RustElementCopy _ | `RustLength _ | `RustRead _
  | `RustWrite _ | `RustArrayOf _ | `RustBoxNew _ | `RustBoxGet _ | `RustBoxSet _ | `RustRetype _ | `RustArrayNew _
  | `RustAllocated _ | `RustTry | `RustThrew | `RustReturned | `RustCaught _ | `RustOutOfBounds _ ->
      false

let unrepeatable node =
  match node with
  | `RustElementCopy _ | `RustArrayOf _ | `RustBoxNew _ | `RustArrayNew _ | `RustTry | `RustCaught _ -> true
  | `RustCast _ | `RustDivide | `RustWrappingRem | `RustWrappingShl _ | `RustWrappingShr _ | `RustSignedWrappingShr _
  | `RustRotateLeft _ | `RustRotateRight _ | `RustLessThan | `RustLessThanEqual | `RustEqual | `RustWrappingAdd
  | `RustWrappingSub | `RustWrappingMul | `RustBitAnd | `RustBitOr | `RustBitXor | `RustRemainder | `RustAdd
  | `RustSubtract | `RustMultiply | `RustNegate | `RustCopysign | `RustAbs | `RustSqrt | `RustCeil | `RustFloor
  | `RustTrunc | `RustRoundTiesEven | `RustFromBits _ | `RustToBits _ | `RustPointerEqual | `RustLeadingZeros
  | `RustTrailingZeros | `RustCountOnes | `RustElementGet _ | `RustElementSet _ | `RustElementFill _ | `RustLength _
  | `RustRead _ | `RustWrite _ | `RustBoxGet _ | `RustBoxSet _ | `RustRetype _ | `RustAllocated _ | `RustThrew
  | `RustReturned | `RustOutOfBounds _ ->
      false

let array_access _ = None

let element_get types array index =
  let array_type = Link.type_ types array in
  Node.create
    (Extension (`RustElementGet (TypeArena.element types array_type, array_type, Link.type_ types index)))
    [| array; index |]

let element_set types array index value =
  Node.create
    (Extension (`RustElementSet (Link.type_ types array, Link.type_ types index, Link.type_ types value)))
    [| array; index; value |]

let element_fill types array start count value =
  Node.create
    (Extension (`RustElementFill (Link.type_ types array, Link.type_ types start, Link.type_ types value)))
    [| array; start; count; value |]

let element_copy types destination at source start count =
  Node.create
    (Extension (`RustElementCopy (Link.type_ types destination, Link.type_ types at)))
    [| destination; at; source; start; count |]

let length types array = Node.create (Extension (`RustLength (Link.type_ types array))) [| array |]

let read types memory offset value_type =
  Node.create
    (Extension (`RustRead (Link.type_ types memory, Link.type_ types offset, value_type)))
    [| memory; offset |]

let write types memory offset value =
  Node.create
    (Extension (`RustWrite (Link.type_ types memory, Link.type_ types offset, Link.type_ types value)))
    [| memory; offset; value |]

let array_new types length initial =
  let initial_type = Link.type_ types initial in
  Node.create
    (Extension (`RustArrayNew (TypeArena.intern types (Array initial_type), initial_type)))
    [| length; initial |]

let allocated types array = Node.create (Extension (`RustAllocated (Link.type_ types array))) [| array |]
let try_ callee arguments = Node.create (Extension `RustTry) (Array.append [| callee |] arguments)
let threw outcome = Node.create (Extension `RustThrew) [| outcome |]
let returned outcome = Node.create (Extension `RustReturned) [| outcome |]
let caught outcome payload_types = Node.create (Extension (`RustCaught payload_types)) [| outcome |]
