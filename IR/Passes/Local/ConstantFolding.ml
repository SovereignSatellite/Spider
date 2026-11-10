open IR
open Definitions

let signed width bits = Int64.shift_right (Int64.shift_left bits (64 - width)) (64 - width)

(** The sign operations act on the sign bit alone, so NaNs keep their payloads. *)
let sign_bit width = Int64.shift_left 1L (Scalar.bits width - 1)

let as_float width bits =
  match width with `F32 -> Int32.float_of_bits (Int64.to_int32 bits) | `F64 -> Int64.float_of_bits bits

let of_float width value =
  match width with
  | `F32 -> Int64.logand (Int64.of_int32 (Int32.bits_of_float value)) 0xFFFF_FFFFL
  | `F64 -> Int64.bits_of_float value

let rec leading_zeros value =
  if value = 0L then 64 else if Int64.compare value 0L < 0 then 0 else 1 + leading_zeros (Int64.shift_left value 1)

let rec trailing_zeros value =
  if Int64.logand value 1L = 1L then 0 else 1 + trailing_zeros (Int64.shift_right_logical value 1)

let rec count_ones value =
  if value = 0L then 0 else Int64.to_int (Int64.logand value 1L) + count_ones (Int64.shift_right_logical value 1)

let rotate_left width value amount =
  Int64.logor (Int64.shift_left value amount) (Int64.shift_right_logical value ((width - amount) mod width))

let minimum width = Int64.neg (Int64.shift_left 1L (width - 1))

let integer_binary (operator : IntegerBinary.t) integer left right =
  let width = Scalar.bits integer in
  let amount = Int64.to_int (Int64.unsigned_rem right (Int64.of_int width)) in
  match operator with
  | Add -> Int64.add left right
  | Subtract -> Int64.sub left right
  | Multiply -> Int64.mul left right
  | And -> Int64.logand left right
  | Or -> Int64.logor left right
  | ExclusiveOr -> Int64.logxor left right
  | ShiftLeft -> Int64.shift_left left amount
  | ShiftRightS -> Int64.shift_right (signed width left) amount
  | ShiftRightU -> Int64.shift_right_logical left amount
  | RotateLeft -> rotate_left width left amount
  | RotateRight -> rotate_left width left ((width - amount) mod width)
  | (DivideS | DivideU | RemainderS | RemainderU) when right = 0L -> 0L
  | DivideS -> Int64.div (signed width left) (signed width right)
  | DivideU -> Int64.unsigned_div left right
  | RemainderS -> Int64.rem (signed width left) (signed width right)
  | RemainderU -> Int64.unsigned_rem left right

let integer_count (operator : IntegerCount.t) integer value =
  let width = Scalar.bits integer in
  match operator with
  | LeadingZeros -> Int64.of_int (leading_zeros value - (64 - width))
  | TrailingZeros when value = 0L -> Int64.of_int width
  | TrailingZeros -> Int64.of_int (trailing_zeros value)
  | CountOnes -> Int64.of_int (count_ones value)

let integer_comparison (operator : IntegerComparison.t) integer left right =
  let width = Scalar.bits integer in
  match operator with
  | Equal -> left = right
  | LessThanS -> Int64.compare (signed width left) (signed width right) < 0
  | LessThanU -> Int64.unsigned_compare left right < 0

let copy_sign width left right =
  Int64.logor (Int64.logand left (Int64.lognot (sign_bit width))) (Int64.logand right (sign_bit width))

let number_binary width (operator : NumberBinary.t) left right =
  let compute operation = of_float width (operation (as_float width left) (as_float width right)) in
  match operator with
  | Add -> compute ( +. )
  | Subtract -> compute ( -. )
  | Multiply -> compute ( *. )
  | Divide -> compute ( /. )
  | Minimum -> compute Float.min
  | Maximum -> compute Float.max
  | CopySign -> copy_sign width left right

let negate width value = Int64.logxor value (sign_bit width)
let absolute width value = Int64.logand value (Int64.lognot (sign_bit width))

let nearest value =
  if Float.abs (value -. Float.trunc value) = 0.5 then 2. *. Float.round (value /. 2.) else Float.round value

let number_unary width (operator : NumberUnary.t) value =
  let compute operation = of_float width (operation (as_float width value)) in
  match operator with
  | Negate -> negate width value
  | Absolute -> absolute width value
  | SquareRoot -> compute Float.sqrt
  | RoundUp -> compute Float.ceil
  | RoundDown -> compute Float.floor
  | Truncate -> compute Float.trunc
  | Nearest -> compute nearest

let number_comparison (operator : NumberComparison.t) left right =
  match operator with Equal -> left = right | LessThan -> left < right | LessThanEqual -> left <= right

type signedness = Signed | Unsigned

let bounds target signedness =
  let width = Scalar.bits target in
  match signedness with
  | Signed -> (Int64.to_float (minimum width), -.Int64.to_float (minimum width))
  | Unsigned -> (0.0, Float.pow 2.0 (float_of_int width))

let two_to_63 = 9223372036854775808.0

let whole_bits signedness whole =
  match signedness with
  | Signed -> Int64.of_float whole
  | Unsigned when whole >= two_to_63 -> Int64.add (Int64.of_float (whole -. two_to_63)) Int64.min_int
  | Unsigned -> Int64.of_float whole

let greatest target signedness =
  let mask = Constant.mask (Type.of_scalar target) in
  match signedness with Signed -> Int64.shift_right_logical mask 1 | Unsigned -> mask

let saturated target signedness value =
  let least, past_greatest = bounds target signedness in
  if Float.is_nan value then 0L
  else if value <= least then whole_bits signedness least
  else if value >= past_greatest then greatest target signedness
  else whole_bits signedness (Float.trunc value)

(** The double holding the unsigned value with its bits below the 53 most significant folded into the lowest, exactly,
    so rounding it to 24 bits rounds as the value itself would. *)
let single_rounding magnitude =
  let length = 64 - leading_zeros magnitude in
  if length <= 53 then Int64.to_float magnitude
  else
    let dropped = length - 53 in
    let low = Int64.logand magnitude (Int64.pred (Int64.shift_left 1L dropped)) in
    let kept = Int64.logor (Int64.shift_right_logical magnitude dropped) (if low = 0L then 0L else 1L) in
    Float.ldexp (Int64.to_float kept) dropped

let unsigned_to_float value =
  if Int64.compare value 0L >= 0 then Int64.to_float value
  else Int64.to_float (Int64.logor (Int64.shift_right_logical value 1) (Int64.logand value 1L)) *. 2.0

let convert_integer target signedness value =
  let kind =
    match (target, signedness) with
    | `F32, Signed when Int64.compare value 0L >= 0 -> F32Constant (Int32.bits_of_float (single_rounding value))
    | `F32, Signed -> F32Constant (Int32.bits_of_float (-.single_rounding (Int64.neg value)))
    | `F32, Unsigned -> F32Constant (Int32.bits_of_float (single_rounding value))
    | `F64, Signed -> F64Constant (Int64.to_float value)
    | `F64, Unsigned -> F64Constant (unsigned_to_float value)
  in
  Link.output (Node.create kind [||]) 0

let conversion (conversion : Conversion.t) bits =
  let constant target = Constant.of_bits (Type.of_scalar target) in
  match conversion with
  | Widen `F32 -> constant `F64 (of_float `F64 (as_float `F32 bits))
  | Narrow `F32 -> constant `F32 (of_float `F32 (as_float `F64 bits))
  | Widen (`I8 | `I16 | `I32) | Narrow (`I8 | `I16 | `I32) | Retype _ -> constant (Conversion.target conversion) bits
  | SignExtend (width, from) -> constant width (signed (Scalar.bits from) bits)
  | ConvertSigned target -> convert_integer target Signed bits
  | ConvertUnsigned target -> convert_integer target Unsigned bits
  | SaturateSigned target -> constant target (saturated target Signed (as_float `F64 bits))
  | SaturateUnsigned target -> constant target (saturated target Unsigned (as_float `F64 bits))

let both inputs =
  match (Constant.bits inputs.(0), Constant.bits inputs.(1)) with
  | Some left, Some right -> Some (left, right)
  | _ -> None

let fold_sum_from_i32 types node inputs variants =
  match Constant.bits inputs.(0) with
  | Some tag when List.mem (Int64.to_int tag) variants ->
      Some
        (Link.output
           (Node.create
              (SumNew { sum_type = Node.output_type types node 0; variant = Int64.to_int tag })
              (Array.sub inputs 1 (Array.length inputs - 1)))
           0)
  | _ -> None

let fold_i32_from_sum inputs =
  match Sums.built inputs.(0).node with
  | Some [ (variant, _) ] -> Some (Link.output (Node.create (I32Constant variant) [||]) 0)
  | _ -> None

let integers = Options.optimization "fold-integer-arithmetic" Integer Folding 1 "Fold integer arithmetic and counts."

let integer_comparisons =
  Options.optimization "fold-integer-comparisons" Integer Comparisons 1 "Fold integer comparisons."

let numbers = Options.optimization "fold-number-arithmetic" Number Folding 1 "Fold F32 and F64 arithmetic."
let number_comparisons = Options.optimization "fold-number-comparisons" Number Folding 1 "Fold F32 and F64 comparisons."
let conversions = Options.optimization "fold-conversions" Conversion Folding 1 "Fold conversions."
let sums = Options.optimization "fold-sums" Sum Folding 1 "Fold a sum of a constant tag and a constant sum's tag."

let folded options types node =
  let inputs = node.inputs in
  let unary optimization fold =
    Options.applied options optimization (fun () -> Option.map fold (Constant.bits inputs.(0)))
  in
  let binary optimization fold =
    Options.applied options optimization (fun () -> Option.map (fun (left, right) -> fold left right) (both inputs))
  in
  let constant width bits = Constant.of_bits (Type.of_scalar width) bits in
  match node.kind with
  | Operator (IntegerBinary (width, operator)) ->
      binary integers (fun left right -> constant width (integer_binary operator width left right))
  | Operator (IntegerCount (width, operator)) ->
      unary integers (fun value -> constant width (integer_count operator width value))
  | Operator (IntegerComparison (width, operator)) ->
      binary integer_comparisons (fun left right -> Constant.boolean (integer_comparison operator width left right))
  | Operator (NumberBinary (width, operator)) ->
      binary numbers (fun left right -> constant width (number_binary width operator left right))
  | Operator (NumberUnary (width, operator)) ->
      unary numbers (fun value -> constant width (number_unary width operator value))
  | Operator (NumberComparison (width, operator)) ->
      binary number_comparisons (fun left right ->
          Constant.boolean (number_comparison operator (as_float width left) (as_float width right)))
  | Operator (Conversion operator) -> unary conversions (conversion operator)
  | SumFromI32 { variants; _ } -> Options.applied options sums (fun () -> fold_sum_from_i32 types node inputs variants)
  | I32FromSum -> Options.applied options sums (fun () -> fold_i32_from_sum inputs)
  | _ -> None

let fold options types node = Option.map (fun link -> [| link |]) (folded options types node)
