open IR
open Definitions

let commutative_order =
  Options.optimization "order-commutative-operands" Graph Sharing 1
    "Move a commutative operation's constant to its right."

let integer_arithmetic =
  Options.optimization "simplify-integer-arithmetic" Integer Identities 1
    "Remove integer operations by 0, 1 or all ones."

let extension_masks =
  Options.optimization "mask-extensions" Integer Identities 1 "Remove a sign extension a mask clears."

let tag_masks = Options.optimization "mask-tags" Integer Identities 1 "Remove a mask keeping every bit of a sum's tag."

let constant_reassociation =
  Options.optimization "reassociate-integer-constants" Integer Identities 1
    "Combine the constants of nested operations."

let narrowed_arithmetic =
  Options.optimization "narrow-integer-arithmetic" Integer Identities 1
    "Narrow an I64 operation to I32 where only its low half is read."

let integer_comparisons =
  Options.optimization "simplify-integer-comparisons" Integer Comparisons 1
    "Fold comparisons of a value with itself, 0 or 1, and a difference's with 0."

let extension_comparisons =
  Options.optimization "compare-through-extensions" Integer Comparisons 1 "Compare a value before its extension."

let inverse_comparisons =
  Options.optimization "compare-through-inverses" Integer Comparisons 1
    "Compare a value before an offset or exclusive or."

let number_arithmetic =
  Options.optimization "simplify-number-arithmetic" Number Identities 1
    "Remove exact F32 and F64 operations by 0 or 1 and repeated unary ones."

let cancelled_conversions =
  Options.optimization "cancel-conversions" Conversion Identities 1 "Cancel a conversion undoing another."

let retypes =
  Options.optimization "collapse-retypes" Conversion Identities 1 "Collapse a retype to its original or one retype."

let sums = Options.optimization "simplify-sums" Sum Identities 1 "Cancel a sum of a tag and the tag of a sum."

let unchanging_writes =
  Options.optimization "drop-unchanging-writes" Storage Identities 1 "Drop a write of a type's only value."

let relations =
  Options.optimization "relate-comparisons" Sum Identities 1 "Combine comparisons of the same operands into one."

let is value link = Constant.bits link = Some value
let mask width = Constant.mask (Type.of_scalar width)
let constant width bits = Constant.of_bits (Type.of_scalar width) bits
let is_ones width link = is (mask width) link
let output node = Link.output node 0
let copy node inputs = output (Copying.node node inputs)
let built operator left right = output (Node.create (Operator operator) [| left; right |])
let equals_zero width link = built (IntegerComparison (width, Equal)) link (constant width 0L)

let shifts_by_whole_widths width right =
  match Constant.bits right with
  | Some amount -> Int64.unsigned_rem amount (Int64.of_int (Scalar.bits width)) = 0L
  | None -> false

let integer_binary width (operator : IntegerBinary.t) left right =
  match operator with
  | (Or | ExclusiveOr) when is 0L right -> Some left
  | (Subtract | ExclusiveOr) when Link.equal left right -> Some (constant width 0L)
  | (Multiply | And) when is 0L right -> Some (constant width 0L)
  | Multiply when is 1L right -> Some left
  | (And | Or) when Link.equal left right -> Some left
  | And when is_ones width right -> Some left
  | Or when is_ones width right -> Some right
  | (ShiftLeft | ShiftRightS | ShiftRightU | RotateLeft | RotateRight)
    when shifts_by_whole_widths width right || is 0L left ->
      Some left
  | (ShiftRightS | RotateLeft | RotateRight) when is_ones width left -> Some left
  | (DivideS | DivideU) when is 1L right -> Some left
  | (RemainderS | RemainderU) when is 1L right -> Some (constant width 0L)
  | RemainderS when is_ones width right -> Some (constant width 0L)
  | Add | Subtract | Multiply | And | Or | ExclusiveOr | ShiftLeft | ShiftRightS | ShiftRightU | RotateLeft
  | RotateRight | DivideS | DivideU | RemainderS | RemainderU ->
      None

let less_than_u width = built (IntegerComparison (width, LessThanU))

let difference link =
  match link.node.kind with
  | Operator (IntegerBinary (_, (Subtract | ExclusiveOr))) -> Some (link.node.inputs.(0), link.node.inputs.(1))
  | _ -> None

let integer_comparison width (operator : IntegerComparison.t) left right =
  match operator with
  | _ when Link.equal left right -> Some (Constant.boolean (operator = Equal))
  | LessThanU when is 0L right -> Some (Constant.boolean false)
  | LessThanU when is 1L right -> Some (equals_zero width left)
  | LessThanS when is 0L right ->
      Some (less_than_u width (constant width (Int64.shift_right_logical (mask width) 1)) left)
  | Equal when is 0L right ->
      Option.map (fun (first, second) -> built (IntegerComparison (width, Equal)) first second) (difference left)
  | Equal | LessThanS | LessThanU -> None

let extension link =
  match link.node.kind with
  | Operator (Conversion (SignExtend (_, from))) -> Some (link.node.inputs.(0), Scalar.bits from)
  | _ -> None

let masks_extension width (operator : IntegerBinary.t) left right =
  match (operator, extension left, Constant.bits right) with
  | And, Some (value, bits), Some mask when Int64.shift_right_logical mask bits = 0L ->
      Some (built (IntegerBinary (width, And)) value right)
  | _ -> None

let tag_bits types link =
  match link.node.kind with
  | I32FromSum ->
      let variants = TypeArena.variants types (Node.operand_type types link.node) in
      let length = 64 - ConstantFolding.leading_zeros (Int64.of_int (List.length variants - 1)) in
      Some (Int64.pred (Int64.shift_left 1L length))
  | _ -> None

let masks_tag types (operator : IntegerBinary.t) left right =
  match (operator, tag_bits types left, Constant.bits right) with
  | And, Some bits, Some mask when Int64.logand bits (Int64.lognot mask) = 0L -> Some left
  | Or, Some bits, Some mask when Int64.logand bits (Int64.lognot mask) = 0L -> Some right
  | _ -> None

let low_bits width bits value =
  match value.node.kind with
  | Operator (Conversion (Widen source)) when Scalar.bits source <= bits -> value
  | _ -> built (IntegerBinary (width, And)) value (constant width (Int64.pred (Int64.shift_left 1L bits)))

let below left right = Int64.unsigned_compare left right < 0

(** The greatest [bits]-wide value whose sign extension is at most the bound: sign extension maps the [bits]-wide
    values, in unsigned order, onto the lowest and the highest values of the wider type, half on each side. *)
let greatest_at_most width bits bound =
  let half = Int64.shift_left 1L (bits - 1) in
  let high = Int64.add (Int64.sub (mask width) half) 1L in
  if below bound half then bound
  else if below bound high then Int64.pred half
  else Int64.add (Int64.sub bound high) half

(** Sign extension keeps signed order, mapping the [bits]-wide values onto those from [-half] to [half - 1], whose order
    is the unsigned order of x's low bits with their sign bit flipped: [c < extend(x)] and [extend(x) < c] compare those
    bits with [c + half], or hold for every x or none where [c] is outside that range. *)
let signed_through_extension width bits value bound extended_right =
  let half = Int64.shift_left 1L (bits - 1) in
  let c = ConstantFolding.signed (Scalar.bits width) bound in
  let flipped = built (IntegerBinary (width, ExclusiveOr)) (low_bits width bits value) (constant width half) in
  let position () = constant width (Int64.add c half) in
  if extended_right then
    if c < Int64.neg half then Constant.boolean true
    else if c >= Int64.pred half then Constant.boolean false
    else less_than_u width (position ()) flipped
  else if c <= Int64.neg half then Constant.boolean false
  else if c >= half then Constant.boolean true
  else less_than_u width flipped (position ())

(** Sign extension keeps unsigned order, so [c < extend(x)] exactly where [greatest_at_most c < x] on x's low bits. *)
let through_extension width (operator : IntegerComparison.t) left right =
  match (operator, Constant.bits left, extension right, extension left, Constant.bits right) with
  | LessThanU, Some bound, Some (value, bits), _, _ ->
      Some (less_than_u width (constant width (greatest_at_most width bits bound)) (low_bits width bits value))
  | LessThanS, Some bound, Some (value, bits), _, _ -> Some (signed_through_extension width bits value bound true)
  | LessThanS, _, _, Some (value, bits), Some bound -> Some (signed_through_extension width bits value bound false)
  | _ -> None

let constant_operand node =
  match node.kind with
  | Operator (IntegerBinary (_, operator)) -> Option.map (fun bits -> (operator, bits)) (Constant.bits node.inputs.(1))
  | _ -> None

let same_offset types offset node =
  Node.same_kind types offset node
  && Link.equal offset.inputs.(0) node.inputs.(0)
  && Constant.bits offset.inputs.(1) = Constant.bits node.inputs.(1)

(** An offset has one shape, [Offsets.offset] of its origin and the sum of its constants: [(x + a) - b] is
    [x + (a - b)], and an offset by zero is its origin. *)
let canonical_offset types node =
  let left = node.inputs.(0) in
  let origin, inner =
    match Offsets.offset_of left.node with Some (_, inner) -> (left.node.inputs.(0), inner) | None -> (left, 0L)
  in
  Option.bind (Offsets.offset_of node) (fun (width, outer) ->
      let offset = Offsets.offset width origin (Int64.logand (Int64.add inner outer) (mask width)) in
      if Link.equal offset origin || not (same_offset types offset.node node) then Some offset else None)

let associative (operator : IntegerBinary.t) =
  match operator with Multiply | And | Or | ExclusiveOr -> true | _ -> false

let reassociated width node =
  let inner = node.inputs.(0).node in
  match (constant_operand node, constant_operand inner) with
  | Some (operator, second), Some (inner_operator, first) when associative operator && inner_operator = operator ->
      let combined = ConstantFolding.integer_binary operator width first second in
      Some (copy node [| inner.inputs.(0); constant width combined |])
  | _ -> None

let inverse inner =
  match (Offsets.offset_of inner, constant_operand inner) with
  | Some (_, offset), _ -> Some (fun value -> Int64.sub value offset)
  | _, Some (ExclusiveOr, bits) -> Some (fun value -> Int64.logxor value bits)
  | _ -> None

let through_inverse width (operator : IntegerComparison.t) node =
  let left = node.inputs.(0) in
  match (inverse left.node, Constant.bits node.inputs.(1)) with
  | Some undo, Some value when operator = Equal ->
      Some (copy node [| left.node.inputs.(0); constant width (undo value) |])
  | _ -> None

let is_number width value link = is (ConstantFolding.of_float width value) link

let number_binary width (operator : NumberBinary.t) left right =
  match operator with
  | Multiply when is_number width 1.0 right -> Some left
  | Divide when is_number width 1.0 right -> Some left
  | Subtract when is_number width 0.0 right -> Some left
  | Add when is_number width (-0.0) right -> Some left
  | (Minimum | Maximum | CopySign) when Link.equal left right -> Some left
  | Divide | Add | Subtract | Multiply | Minimum | Maximum | CopySign -> None

let rounds (operator : NumberUnary.t) =
  match operator with RoundUp | RoundDown | Truncate | Nearest -> true | Negate | Absolute | SquareRoot -> false

let number_unary operator node =
  let operand = node.inputs.(0) in
  match operand.node.kind with
  | Operator (NumberUnary (_, inner)) when inner = operator && operator = Negate -> Some operand.node.inputs.(0)
  | Operator (NumberUnary (_, inner)) when inner = operator && operator = Absolute -> Some operand
  | Operator (NumberUnary (_, inner)) when rounds inner && rounds operator -> Some operand
  | Operator (NumberUnary (_, inner)) when inner = Negate && operator = Absolute ->
      Some (copy node [| operand.node.inputs.(0) |])
  | _ -> None

let tag_of_sum tag =
  let sum = tag.inputs.(0).node in
  match sum.kind with SumFromI32 _ -> Some (SumFromI32.tag sum) | _ -> None

let sum_of_tag types built =
  let tag = SumFromI32.tag built in
  match tag.node.kind with
  | I32FromSum
    when Array.length (SumFromI32.values built) = 0 && Node.operand_type types tag.node = Node.output_type types built 0
    ->
      Some tag.node.inputs.(0)
  | _ -> None

let undoes (conversion : Conversion.t) (inner : Conversion.t) =
  match (conversion, inner) with Narrow _, Widen _ | Retype _, Retype _ -> true | _ -> false

let low_half (operator : IntegerBinary.t) =
  match operator with
  | Add | Subtract | Multiply | And | Or | ExclusiveOr -> Some (built (IntegerBinary (`I32, operator)))
  | ShiftLeft | ShiftRightS | ShiftRightU | RotateLeft | RotateRight | DivideS | DivideU | RemainderS | RemainderU ->
      None

let narrowed inner build =
  build
    (output (Node.create (Operator (Conversion (Narrow `I32))) [| inner.inputs.(0) |]))
    (output (Node.create (Operator (Conversion (Narrow `I32))) [| inner.inputs.(1) |]))

let conversion options readers (conversion : Conversion.t) node =
  let inner = node.inputs.(0).node in
  match (conversion, inner.kind) with
  | _, Operator (Conversion inner_conversion)
    when options cancelled_conversions && undoes conversion inner_conversion
         && Conversion.source inner_conversion = Conversion.target conversion ->
      Some inner.inputs.(0)
  | Narrow `I32, Operator (IntegerBinary (_, inner_operator))
    when options narrowed_arithmetic && Readers.only_by readers readers.region node inner ->
      Option.map (narrowed inner) (low_half inner_operator)
  | _ -> None

let retype types node =
  let input = node.inputs.(0) in
  let value_type = Node.output_type types node 0 in
  let original = if Node.same_kind types input.node node then input.node.inputs.(0) else input in
  if Link.type_ types original = value_type then Some original
  else if not (Link.equal original input) then Some (copy node [| original |])
  else None

let rec one_valued types value_type =
  match TypeArena.definition types value_type with
  | Structural (Sum [ fields ]) -> List.for_all (one_valued types) fields
  | _ -> false

let unchanging types node held =
  let reference = node.inputs.(0) in
  if one_valued types (held types (Link.type_ types reference)) then Some reference else None

let first_of options attempts =
  List.find_map (fun (optimization, attempt) -> Options.applied options optimization attempt) attempts

(** A commutative operation's constant moves to its right, so the rules for those operations test only the right one. *)
let simplified options types readers node =
  let input index = node.inputs.(index) in
  let applied = Options.applied options in
  if
    options commutative_order && Node.commutative types node && Constant.is (input 0).node
    && not (Constant.is (input 1).node)
  then Some (copy node [| input 1; input 0 |])
  else
    match node.kind with
    | Operator (IntegerBinary (width, operator)) ->
        first_of options
          [
            (integer_arithmetic, fun () -> integer_binary width operator (input 0) (input 1));
            (extension_masks, fun () -> masks_extension width operator (input 0) (input 1));
            (tag_masks, fun () -> masks_tag types operator (input 0) (input 1));
            (NodeReduction.canonical_offsets, fun () -> canonical_offset types node);
            (constant_reassociation, fun () -> reassociated width node);
          ]
    | Operator (IntegerComparison (width, operator)) ->
        first_of options
          [
            (integer_comparisons, fun () -> integer_comparison width operator (input 0) (input 1));
            (extension_comparisons, fun () -> through_extension width operator (input 0) (input 1));
            (inverse_comparisons, fun () -> through_inverse width operator node);
          ]
    | Operator (NumberComparison (_, LessThan)) when options number_arithmetic && Link.equal (input 0) (input 1) ->
        Some (Constant.boolean false)
    | Operator (NumberBinary (width, operator)) ->
        applied number_arithmetic (fun () -> number_binary width operator (input 0) (input 1))
    | Operator (NumberUnary (_, operator)) -> applied number_arithmetic (fun () -> number_unary operator node)
    | Operator (Conversion operator) -> conversion options readers operator node
    | BoxRetype _ | FunctionRetype _ -> applied retypes (fun () -> retype types node)
    | I32FromSum -> applied sums (fun () -> tag_of_sum node)
    | SumFromI32 _ -> applied sums (fun () -> sum_of_tag types node)
    | ArraySet _ | ArrayFill _ | ArrayCopy _ ->
        applied unchanging_writes (fun () -> unchanging types node TypeArena.element)
    | BoxSet _ -> applied unchanging_writes (fun () -> unchanging types node TypeArena.contents)
    | _ -> None

let fold options types table node =
  match simplified options types table node with
  | Some link -> Some [| link |]
  | None ->
      Option.map
        (fun link -> [| link |])
        (Options.applied options relations (fun () -> Relations.of_operation types node))
