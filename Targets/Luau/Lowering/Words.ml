open IR
open Definitions
open Passes
open Numbers

(* Luau: an I64 lowers to two I32 words. *)
let words value =
  let split = Node.create (Extension `LuauBitsFromI64) [| value |] in
  (Link.output split 0, Link.output split 1)

let narrow value = fst (words value)
let join low high = extended `LuauBitsIntoI64 [| low; high |]
let word_types = [ Type.i32; Type.i32 ]

let carrying combine left right =
  let left_low, left_high = words left in
  let right_low, right_high = words right in
  let low = combine (to_number left_low) (to_number right_low) in
  let carry = extended `LuauMathFloor [| multiply low (constant (1.0 /. 4294967296.0)) |] in
  let high = Numbers.add (combine (to_number left_high) (to_number right_high)) carry in
  join (to_i32 (Numbers.subtract low (multiply carry (constant 4294967296.0)))) (wrap high)

let add = carrying Numbers.add
let subtract = carrying Numbers.subtract

let bitwise combine left right =
  let left_low, left_high = words left in
  let right_low, right_high = words right in
  join (combine (to_number left_low) (to_number right_low)) (combine (to_number left_high) (to_number right_high))

(** The words join after the Match, not in each branch, so a reader splitting the result again takes the Match's words.
*)
let shift types near far value amount =
  let low, high = words value in
  let reduced = extended `LuauBit32Band [| to_number (narrow amount); constant 63.0 |] in
  let shifted =
    Branch.choose types
      (less_than_equal (constant 32.0) (to_number reduced))
      [| low; high; reduced |] word_types
      (fun fields -> near fields.(0) fields.(1) fields.(2))
      (fun fields -> far fields.(0) fields.(1) fields.(2))
  in
  join shifted.(0) shifted.(1)

let beyond amount = Numbers.subtract (to_number amount) (constant 32.0)
let bitwise_or left right = extended `LuauBit32Bor [| to_number left; to_number right |]

(** [kept] shifted by an amount below 32, with the bits [incoming] carries across words: [incoming] shifted the other
    way by one, then by 31 less the amount, so no shift reaches 32 and an amount of zero carries none. *)
let spliced shift opposite kept incoming amount =
  let shifted = extended shift [| to_number kept; to_number amount |] in
  let once = extended opposite [| to_number incoming; constant 1.0 |] in
  bitwise_or shifted (extended opposite [| to_number once; Numbers.subtract (constant 31.0) (to_number amount) |])

let shift_left_near low high amount =
  [|
    extended `LuauBit32Lshift [| to_number low; to_number amount |];
    spliced `LuauBit32Lshift `LuauBit32Rshift high low amount;
  |]

let shift_left_far low _ amount = [| integer 0; extended `LuauBit32Lshift [| to_number low; beyond amount |] |]

let shift_right_near shift low high amount =
  [| spliced `LuauBit32Rshift `LuauBit32Lshift low high amount; extended shift [| to_number high; to_number amount |] |]

let shift_right_u_far _ high amount = [| extended `LuauBit32Rshift [| to_number high; beyond amount |]; integer 0 |]

let shift_right_s_far _ high amount =
  [|
    extended `LuauBit32Arshift [| to_number high; beyond amount |];
    extended `LuauBit32Arshift [| to_number high; constant 31.0 |];
  |]

let rotate types turn value amount =
  let low, high = words value in
  let amount_low = narrow amount in
  let below_half =
    equal (to_number (extended `LuauBit32Band [| to_number amount_low; constant 32.0 |])) (constant 0.0)
  in
  let swapped =
    Branch.choose types below_half [| low; high |] word_types (fun fields -> [| fields.(1); fields.(0) |]) Fun.id
  in
  let rest = extended `LuauBit32Band [| to_number amount_low; constant 31.0 |] in
  join (turn swapped.(0) swapped.(1) rest) (turn swapped.(1) swapped.(0) rest)

let halves word =
  let upper = extended `LuauFloorDivide [| to_number word; constant 65536.0 |] in
  (Numbers.subtract (to_number word) (multiply upper (constant 65536.0)), upper)

let shifted value = to_number (extended `LuauBit32Lshift [| value; constant 16.0 |])

(** The low words' rounded product is within 2^11 of the exact one, so the scaled difference from its low word is within
    2^-21 of the carry, which adding 1.5 * 2^52, a multiple of 2^32, rounds to. *)
let product left right =
  let left_low, left_high = words left in
  let right_low, right_high = words right in
  let right_low_lower, right_low_upper = halves right_low in
  let right_high_lower, right_high_upper = halves right_high in
  let left_low, left_high = (to_number left_low, to_number left_high) in
  let low = wrap (Numbers.add (multiply right_low_lower left_low) (shifted (multiply right_low_upper left_low))) in
  let cross =
    Numbers.add
      (Numbers.add (multiply right_high_lower left_low) (multiply right_low_lower left_high))
      (shifted (Numbers.add (multiply right_high_upper left_low) (multiply right_low_upper left_high)))
  in
  let difference = Numbers.subtract (multiply left_low (to_number right_low)) (to_number low) in
  let carry = multiply difference (constant (1.0 /. 4294967296.0)) in
  join low (wrap (Numbers.add (Numbers.add carry (constant 6755399441055744.0)) cross))

let binary types (operator : IntegerBinary.t) left right =
  (* Luau: an I64 lowers to two I32 words. *)
  let words_with operation left right = extended operation [| left; right |] in
  match operator with
  | Add -> Some (add left right)
  | Subtract -> Some (subtract left right)
  | Multiply -> Some (product left right)
  | And -> Some (bitwise (words_with `LuauBit32Band) left right)
  | Or -> Some (bitwise (words_with `LuauBit32Bor) left right)
  | ExclusiveOr -> Some (bitwise (words_with `LuauBit32Bxor) left right)
  | ShiftLeft -> Some (shift types shift_left_near shift_left_far left right)
  | ShiftRightS -> Some (shift types (shift_right_near `LuauBit32Arshift) shift_right_s_far left right)
  | ShiftRightU -> Some (shift types (shift_right_near `LuauBit32Rshift) shift_right_u_far left right)
  | RotateLeft -> Some (rotate types (spliced `LuauBit32Lshift `LuauBit32Rshift) left right)
  | RotateRight -> Some (rotate types (spliced `LuauBit32Rshift `LuauBit32Lshift) left right)
  | DivideS | DivideU | RemainderS | RemainderU -> None

let count_zeros types count tested other =
  let counted =
    Branch.choose_one types
      (equal (to_number tested) (constant 0.0))
      [| tested; other |] Type.i32
      (fun fields -> count fields.(0))
      (fun fields -> to_i32 (Numbers.add (to_number (count fields.(1))) (constant 32.0)))
  in
  join counted (integer 0)

let count types (operator : IntegerCount.t) value =
  let low, high = words value in
  match operator with
  | LeadingZeros -> count_zeros types (fun word -> extended `LuauBit32Countlz [| to_number word |]) high low
  | TrailingZeros -> count_zeros types (fun word -> extended `LuauBit32Countrz [| to_number word |]) low high
  | CountOnes ->
      join
        (to_i32 (Numbers.add (to_number (Integers.count_ones low)) (to_number (Integers.count_ones high))))
        (integer 0)

let order types order_high left right =
  let left_low, left_high = words left in
  let right_low, right_high = words right in
  Branch.choose_one types
    (equal (to_number left_high) (to_number right_high))
    [| left_low; left_high; right_low; right_high |]
    Type.boolean
    (fun fields -> less_than (to_number (order_high fields.(1))) (to_number (order_high fields.(3))))
    (fun fields -> less_than (to_number fields.(0)) (to_number fields.(2)))

let equal_words types left right =
  let left_low, left_high = words left in
  let right_low, right_high = words right in
  Branch.both types
    (equal (to_number left_low) (to_number right_low))
    (equal (to_number left_high) (to_number right_high))

let compare types (operator : IntegerComparison.t) left right =
  match operator with
  (* Luau: an I64 lowers to two I32 words. *)
  | Equal -> equal_words types left right
  | LessThanS -> order types flip_sign left right
  | LessThanU -> order types Fun.id left right

let widen value = join value (integer 0)

let sign_extend bits value =
  (* Luau: an I64 lowers to two I32 words. *)
  let low = narrow value in
  let extended_low = if bits = 32 then low else Integers.sign_extend bits low in
  join extended_low (extended `LuauBit32Arshift [| to_number extended_low; constant 31.0 |])

let convert high value =
  let low, high_word = words value in
  Numbers.add (to_number low) (multiply (high high_word) (constant 4294967296.0))

let truncated_signed value =
  let low = wrap value in
  [| low; wrap (multiply (Numbers.subtract value (to_number low)) (constant (1.0 /. 4294967296.0))) |]

let truncated_unsigned value =
  [|
    wrap (extended `LuauModulo [| value; constant 4294967296.0 |]);
    to_i32 (extended `LuauFloorDivide [| value; constant 4294967296.0 |]);
  |]
