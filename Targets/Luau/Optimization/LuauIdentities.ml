open IR
open Definitions
open Passes
open LuauExtension
open LuauValues

let is_zero link = modular_constant link = Some 0
let is_positive_zero link = Constant.bits link = Some 0L
let ranged bounds link = Option.is_some (LuauBounds.range bounds link)

let non_negative bounds link =
  match LuauBounds.range bounds link with Some range -> range.least >= 0.0 | None -> false

let copied node inputs = Link.output (Copying.node node inputs) 0

let as_word bounds value =
  match (value.node.kind, LuauBounds.range bounds value) with
  | Extension `LuauToNumber, _ when Link.type_ (LuauBounds.types bounds) value.node.inputs.(0) = Type.i32 ->
      Some value.node.inputs.(0)
  (* Luau: undiagnosed; converge or justify. *)
  | _, Some range when range.least >= 0.0 && range.greatest < 4294967296.0 -> Some (Numbers.of_number `I32 value)
  | _ -> None

let commutative node =
  match node.kind with Extension (`LuauBit32Band | `LuauBit32Bor | `LuauBit32Bxor) -> true | _ -> false

let with_constant node =
  match (modular_constant node.inputs.(1), modular_constant node.inputs.(0)) with
  | Some constant, _ -> Some (node.inputs.(0), constant)
  | None, Some constant when commutative node -> Some (node.inputs.(1), constant)
  | _ -> None

let keeping low link =
  let node = link.node in
  match (node.kind, if commutative node then with_constant node else None) with
  | Extension `LuauBit32Band, Some (operand, constant) when constant land low = low -> Some operand
  | Extension (`LuauBit32Bor | `LuauBit32Bxor), Some (operand, constant) when constant land low = 0 -> Some operand
  | Extension `LuauWrap, _ -> Some node.inputs.(0)
  | _ -> None

let wrap bounds value = match as_word bounds value with Some _ as word -> word | None -> Some (Numbers.wrap value)

let fused_constant node inner outer =
  match node.kind with
  | Extension `LuauBit32Band -> Some (inner land outer)
  | Extension `LuauBit32Bor -> Some (inner lor outer)
  | Extension `LuauBit32Bxor -> Some (inner lxor outer)
  (* Luau: undiagnosed; converge or justify. *)
  | Extension (`LuauBit32Lshift | `LuauBit32Rshift) -> Some (min (inner + outer) 32)
  | Extension `LuauBit32Arshift -> Some (min (inner + outer) 31)
  | Extension (`LuauBit32Lrotate | `LuauBit32Rrotate) -> Some ((inner + outer) mod 32)
  | _ -> None

let fused_binary types node inner outer =
  match (with_constant inner, node.kind, inner.kind) with
  | Some (source, constant), _, _ when Node.same_kind types inner node ->
      Option.map
        (fun fused -> copied node [| source; Numbers.constant (float_of_int fused) |])
        (fused_constant node constant outer)
  | Some (source, constant), Extension `LuauBit32Band, Extension (`LuauBit32Bor | `LuauBit32Bxor)
    when constant land outer = 0 ->
      Some (copied node [| source; Numbers.constant (float_of_int outer) |])
  | _ -> None

let fused_operand types node operand outer =
  let inner = operand.node.inputs.(0).node in
  match inner.kind with
  | Extension
      ( `LuauBit32Band | `LuauBit32Bor | `LuauBit32Bxor | `LuauBit32Lshift | `LuauBit32Rshift | `LuauBit32Arshift
      | `LuauBit32Lrotate | `LuauBit32Rrotate ) ->
      fused_binary types node inner outer
  | _ -> None

let fused types node =
  match with_constant node with
  | Some (({ node = { kind = Extension `LuauToNumber; _ }; _ } as operand), outer) ->
      fused_operand types node operand outer
  | _ -> None

let replaced replacement node =
  let inputs = node.inputs in
  let replacements = Array.map replacement inputs in
  if Array.for_all Option.is_none replacements then None
  else
    Some
      (copied node (Array.map2 (fun input replacement -> Option.value replacement ~default:input) inputs replacements))

(** A left shift reads its operand modulo 2^32, so modulo 2^32 the negation of its result is the shift of its operand's
    (* Luau: undiagnosed; converge or justify. *) negation, unless an amount from 2^31 shifts right. *)
let negated_shift bounds operand =
  match (operand.node.kind, operand.node.inputs) with
  | ( Extension `LuauSubtract,
      [| zero; { node = { kind = Extension `LuauToNumber; inputs = [| { node = shift; _ } |]; _ }; _ } |] )
    when number zero = Some 0.0 -> (
      match (shift.kind, shift.inputs) with
      (* Luau: undiagnosed; converge or justify. *)
      | Extension `LuauBit32Lshift, [| shifted; amount |] when LuauBounds.operand_ones bounds amount < 0x8000_0000 ->
          Some (Numbers.to_number (copied shift [| Numbers.subtract zero shifted; amount |]))
      | _ -> None)
  | _ -> None

(* Luau: undiagnosed; converge or justify. *)

(** A consumer taking its operand modulo 2^width, as [bit32] and the buffer's integer writes do, needs only its [low]
    bits, so an inner [bit32] operation keeping them can go, also from a term of an exact sum or difference. *)
let narrowed bounds low operand =
  let term link = match link.node.kind with Extension `LuauToNumber -> keeping low link.node.inputs.(0) | _ -> None in
  match operand.node.kind with
  | Extension `LuauToNumber -> term operand
  | Extension (`LuauAdd | `LuauSubtract) -> (
      (* Luau: undiagnosed; converge or justify. *)
      match (negated_shift bounds operand, replaced term operand.node) with
      | (Some _ as shift), _ -> shift
      | None, Some sum when ranged bounds operand && ranged bounds sum -> Some sum
      | _ -> None)
  | _ -> None

let is_ones link = modular_constant link = Some ones

let reduced bounds node =
  let left, right = (node.inputs.(0), node.inputs.(1)) in
  let left_ones = LuauBounds.operand_ones bounds left in
  let right_ones = LuauBounds.operand_ones bounds right in
  let is_constant link = Option.is_some (modular_constant link) in
  match node.kind with
  | _ when LuauBounds.ones bounds (Link.output node 0) = 0 -> Some (Numbers.integer 0)
  | Extension `LuauBit32Bxor when Link.equal left right -> Some (Numbers.integer 0)
  | Extension (`LuauBit32Band | `LuauBit32Bor) when Link.equal left right -> wrap bounds left
  | Extension `LuauBit32Band when left_ones land lnot right_ones = 0 && is_constant right -> wrap bounds left
  | Extension `LuauBit32Band when right_ones land lnot left_ones = 0 && is_constant left -> wrap bounds right
  | Extension (`LuauBit32Bor | `LuauBit32Bxor) when right_ones = 0 -> wrap bounds left
  | Extension (`LuauBit32Bor | `LuauBit32Bxor) when left_ones = 0 -> wrap bounds right
  | Extension `LuauBit32Bor when left_ones land lnot right_ones = 0 && is_constant right ->
      Some (Numbers.integer right_ones)
  | Extension `LuauBit32Bor when right_ones land lnot left_ones = 0 && is_constant left ->
      Some (Numbers.integer left_ones)
  | Extension (`LuauBit32Band | `LuauBit32Bor | `LuauBit32Bxor) -> None
  | _ when is_zero right -> wrap bounds left
  | Extension (`LuauBit32Arshift | `LuauBit32Lrotate | `LuauBit32Rrotate) when is_ones left ->
      Some (Numbers.integer ones)
  | _ -> None

let added bounds node =
  match (node.kind, with_constant node) with
  | Extension (`LuauBit32Bor | `LuauBit32Bxor), Some (operand, constant) when constant <> 0 -> (
      match LuauBounds.word_ones bounds operand with
      | Some ones when ones land constant = 0 ->
          (* Luau: undiagnosed; converge or justify. *)
          Some (Numbers.of_number `I32 (Numbers.add operand (Numbers.constant (float_of_int constant))))
      | _ -> None)
  | _ -> None

(** Every bit of a word shifted arithmetically by 31 is its sign bit, so the word is 0 or 2^32 - 1, and an operation
    with a constant on it is the operation at 0 plus the sign bit times the step from there to the operation at 2^32 -
    1. *)
let spread node =
  match with_constant node with
  | Some ({ node = { kind = Extension `LuauToNumber; inputs = [| splat |]; _ }; _ }, constant) -> (
      match (splat.node, bit32_binary node 0 constant, bit32_binary node ones constant) with
      | { kind = Extension `LuauBit32Arshift; inputs = [| _; amount |]; _ }, Some 0, Some set
        when modular_constant amount = Some 31 && set = ones ->
          Some splat
      | { kind = Extension `LuauBit32Arshift; inputs = [| source; amount |]; _ }, Some cleared, Some set
        when modular_constant amount = Some 31 ->
          let sign = Numbers.to_number (Numbers.extended `LuauBit32Rshift [| source; amount |]) in
          let step = Numbers.multiply sign (Numbers.constant (float_of_int (abs (set - cleared)))) in
          let at_zero = Numbers.constant (float_of_int cleared) in
          Some
            (Numbers.of_number `I32 (if set < cleared then Numbers.subtract at_zero step else Numbers.add step at_zero))
      | _ -> None)
  | _ -> None

let shift_of link =
  match (link.node.kind, link.node.inputs) with
  | ( Extension `LuauToNumber,
      [|
        {
          node =
            {
              kind =
                Extension
                  (`LuauBit32Lshift | `LuauBit32Rshift | `LuauBit32Arshift | `LuauBit32Lrotate | `LuauBit32Rrotate);
              _;
            } as shift;
          _;
        };
      |] ) ->
      Some shift
  | _ -> None

(** Each bit a shift or rotation gives is one bit of its operand or a fill, alike for both operands where the amount is,
    so a bitwise operation of two such is the shift of the operation. *)
let distributed types node =
  match (commutative node, shift_of node.inputs.(0), shift_of node.inputs.(1)) with
  | true, Some left, Some right when Node.same_kind types left right && Link.equal left.inputs.(1) right.inputs.(1) ->
      let combined = copied node [| left.inputs.(0); right.inputs.(0) |] in
      Some (copied left [| Numbers.to_number combined; left.inputs.(1) |])
  | _ -> None

(* Luau: undiagnosed; converge or justify. *)
let rec viewed value =
  match value.node.kind with
  | Extension (`LuauToNumber | `LuauFromNumber _) -> viewed value.node.inputs.(0)
  | _ -> value

let is_left_shift amount node =
  match node.kind with Extension `LuauBit32Lshift -> modular_constant node.inputs.(1) = Some amount | _ -> false

let is_right_shift amount node =
  match node.kind with Extension `LuauBit32Arshift -> modular_constant node.inputs.(1) = Some amount | _ -> false

(** A left shift keeps only its operand's low bits. *)
let shift_narrowed bounds node =
  match (node.kind, modular_constant node.inputs.(1)) with
  | Extension `LuauBit32Lshift, Some amount ->
      Option.map
        (fun operand -> copied node [| operand; node.inputs.(1) |])
        (narrowed bounds (ones lsr amount) node.inputs.(0))
  | _ -> None

(** Shifting up and arithmetically back leaves a value its field holds, as a word. *)
let sign_kept bounds node =
  match (node.kind, modular_constant node.inputs.(1), (viewed node.inputs.(0)).node) with
  | Extension `LuauBit32Arshift, Some amount, shift when is_left_shift amount shift -> (
      let limit = Float.ldexp 1.0 (31 - amount) in
      match LuauBounds.range bounds shift.inputs.(0) with
      | Some range when -.limit <= range.least && range.greatest < limit -> Some (Numbers.wrap shift.inputs.(0))
      | _ -> None)
  | _ -> None

let bitwise bounds node =
  List.find_map
    (fun rule -> rule node)
    [
      reduced bounds;
      added bounds;
      fused (LuauBounds.types bounds);
      (* Luau: undiagnosed; converge or justify. *)
      distributed (LuauBounds.types bounds);
      spread;
      shift_narrowed bounds;
      sign_kept bounds;
      replaced (narrowed bounds ones);
    ]

let unwrapped bounds node =
  match as_word bounds node.inputs.(0) with Some _ as word -> word | None -> replaced (narrowed bounds ones) node

let interval bounds link =
  match (LuauBounds.range bounds link, number link) with
  | Some range, _ -> Some (range.least, range.greatest, true)
  | None, Some value when not (Float.is_nan value) -> Some (value, value, Float.trunc value = value)
  | _ -> None

let compare_intervals (operator : NumberComparison.t) (left_least, left_greatest, left_whole)
    (right_least, right_greatest, right_whole) =
  match operator with
  | LessThan when left_greatest < right_least -> Some true
  | LessThan when left_least >= right_greatest -> Some false
  | LessThanEqual when left_greatest <= right_least -> Some true
  | LessThanEqual when left_least > right_greatest -> Some false
  | Equal when left_greatest < right_least || right_greatest < left_least -> Some false
  | Equal when left_whole <> right_whole -> Some false
  | _ -> None

let compared bounds node operator =
  match (interval bounds node.inputs.(0), interval bounds node.inputs.(1)) with
  | Some left, Some right -> Option.map Constant.boolean (compare_intervals operator left right)
  | _ -> None

(** [source + constant], or [constant - source] where [negated], as a sum or difference with a constant is. *)
type linear = { source : x link; negated : bool; constant : float }

let linear link =
  let inputs = link.node.inputs in
  match link.node.kind with
  | Extension ((`LuauAdd | `LuauSubtract) as operation) -> (
      match (operation, number inputs.(0), number inputs.(1)) with
      | `LuauAdd, _, Some constant -> Some { source = inputs.(0); negated = false; constant }
      | `LuauAdd, Some constant, _ -> Some { source = inputs.(1); negated = false; constant }
      | `LuauSubtract, _, Some constant -> Some { source = inputs.(0); negated = false; constant = 0.0 -. constant }
      | `LuauSubtract, Some constant, _ -> Some { source = inputs.(1); negated = true; constant }
      | _ -> None)
  | _ -> None

let composed outer inner =
  let constant = if outer.negated then 0.0 -. inner.constant else inner.constant in
  { source = inner.source; negated = outer.negated <> inner.negated; constant = constant +. outer.constant }

let built form =
  if Float.abs form.constant >= exact then None
  else if form.negated then Some (Numbers.subtract (Numbers.constant form.constant) form.source)
  else if form.constant = 0.0 then Some form.source
  else Some (Numbers.add form.source (Numbers.constant form.constant))

(** A ranged value is never −0, so [0 - x] is its canonical negation, which ranged sums absorb and ranged products by a
    constant carry outward. *)
let negation link =
  match (link.node.kind, link.node.inputs) with
  | Extension `LuauSubtract, [| zero; source |] when is_positive_zero zero -> Some source
  | _ -> None

let reassociated bounds node =
  match linear (Link.output node 0) with
  | Some outer -> (
      match linear outer.source with
      | Some inner when ranged bounds inner.source -> built (composed outer inner)
      | _ -> None)
  | None -> None

let negated_product negated factor =
  match number factor with
  | Some factor when factor > 0.0 ->
      Some (Numbers.subtract (Numbers.constant 0.0) (Numbers.multiply negated (Numbers.constant factor)))
  | Some factor when factor < 0.0 -> Some (Numbers.multiply negated (Numbers.constant (0.0 -. factor)))
  | _ -> None

(* Luau: undiagnosed; converge or justify. *)
let scaled link =
  let inputs = link.node.inputs in
  match link.node.kind with
  | Extension `LuauMultiply -> (
      match (number inputs.(0), number inputs.(1)) with
      | _, Some factor -> Some (inputs.(0), factor)
      | Some factor, _ -> Some (inputs.(1), factor)
      | _ -> None)
  | _ -> None

(** The value whose bit 31 a word's is, through views, wraps and arithmetic shifts right. *)
let rec sign_source link =
  match (link.node.kind, link.node.inputs) with
  | Extension (`LuauToNumber | `LuauWrap), [| operand |] -> sign_source operand
  | Extension `LuauBit32Arshift, [| operand; amount |]
    when Option.fold ~none:false ~some:(fun bits -> bits < 32) (modular_constant amount) ->
      sign_source operand
  | _ -> link

(** A word below 2^(k + 1) less its bit k times 2^(k + 1) is its low k + 1 bits read signed, as is the word with bit k
    flipped less 2^k. *)
let read_signed bounds node =
  let word = node.inputs.(0) in
  match (node.kind, scaled node.inputs.(1), LuauBounds.word_ones bounds word) with
  | ( Extension `LuauSubtract,
      Some ({ node = { kind = Extension `LuauToNumber; inputs = [| { node = shift; _ } |]; _ }; _ }, factor),
      Some word_ones ) -> (
      match shift with
      | { kind = Extension `LuauBit32Rshift; inputs = [| source; amount |]; _ } -> (
          match modular_constant amount with
          | Some bit
            when bit < 32
                 && (Link.equal source word || (bit = 31 && Link.equal (sign_source source) (sign_source word)))
                 && factor = Float.ldexp 2.0 bit
                 && word_ones lsr (bit + 1) = 0 ->
              let half = Numbers.constant (Float.ldexp 1.0 bit) in
              Some (Numbers.subtract (Numbers.to_number (Numbers.extended `LuauBit32Bxor [| word; half |])) half)
          | _ -> None)
      | _ -> None)
  | _ -> None

let negation_carried (operator : NumberBinary.t) node =
  let left, right = (node.inputs.(0), node.inputs.(1)) in
  match (operator, negation left, negation right) with
  | Add, _, Some negated -> Some (Numbers.subtract left negated)
  | Add, Some negated, _ -> Some (Numbers.subtract right negated)
  | Subtract, _, Some negated -> Some (Numbers.add left negated)
  | Multiply, Some negated, _ -> negated_product negated right
  | Multiply, _, Some negated -> negated_product negated left
  | _ -> None

let arithmetic bounds node (operator : NumberBinary.t) =
  let left, right = (node.inputs.(0), node.inputs.(1)) in
  let ranged_output = ranged bounds (Link.output node 0) in
  match operator with
  | Add when is_positive_zero right && ranged bounds left -> Some left
  | Add when is_positive_zero left && ranged bounds right -> Some right
  | Subtract when is_positive_zero right && ranged bounds left -> Some left
  | Multiply when number right = Some 1.0 && ranged bounds left -> Some left
  | Multiply when number left = Some 1.0 && ranged bounds right -> Some right
  | Subtract when Link.equal left right && ranged bounds left -> Some (Numbers.constant 0.0)
  | Multiply
    when (is_positive_zero right && non_negative bounds left) || (is_positive_zero left && non_negative bounds right) ->
      Some (Numbers.constant 0.0)
  | (Add | Subtract | Multiply) when ranged_output ->
      (* Luau: undiagnosed; converge or justify. *)
      List.find_map (fun rule -> rule node) [ reassociated bounds; negation_carried operator; read_signed bounds ]
  | _ -> None

let rounded bounds node =
  let operand = node.inputs.(0) in
  (* Luau: undiagnosed; converge or justify. *)
  match (node.kind, operand.node.kind) with
  | _ when ranged bounds operand -> Some operand
  | Extension `LuauMathFloor, Extension `LuauDivide ->
      Some (Numbers.extended `LuauFloorDivide [| operand.node.inputs.(0); operand.node.inputs.(1) |])
  | _ -> None

let divided bounds node =
  let left, right = (node.inputs.(0), node.inputs.(1)) in
  let below =
    match (LuauBounds.range bounds left, LuauBounds.range bounds right) with
    | Some dividend, Some divisor -> dividend.least >= 0.0 && dividend.greatest < divisor.least
    | _ -> false
  in
  let by_one = number right = Some 1.0 && ranged bounds left in
  match node.kind with
  | Extension (`LuauModulo | `LuauMathFmod) when below -> Some left
  (* Luau: undiagnosed; converge or justify. *)
  | Extension `LuauFloorDivide when below -> Some (Numbers.constant 0.0)
  | Extension `LuauFloorDivide when by_one -> Some left
  | Extension `LuauModulo when by_one -> Some (Numbers.constant 0.0)
  | Extension `LuauMathFmod when by_one && non_negative bounds left -> Some (Numbers.constant 0.0)
  | _ -> None

(* Luau: undiagnosed; converge or justify. *)
let repacked low high =
  match (low.node.kind, high.node.kind) with
  | Extension `LuauBitsFromI64, Extension `LuauBitsFromI64
    when low.port = 0 && high.port = 1 && Link.equal low.node.inputs.(0) high.node.inputs.(0) ->
      Some low.node.inputs.(0)
  | _ -> None

let among_words bounds word number =
  match LuauBounds.range bounds number with
  | Some range -> range.least >= 0.0 && range.greatest <= Int64.to_float (Constant.mask word)
  | None -> false

(** A word's number converted back collapses only where its range already fits the word, since otherwise the conversion
    is what gives the bounds that range. *)
let viewed_back bounds node input =
  let types = LuauBounds.types bounds in
  match (node.kind, input.node.kind) with
  | Extension (`LuauFromNumber _), Extension `LuauToNumber ->
      Node.operand_type types input.node = Node.output_type types node 0
  | Extension `LuauToNumber, Extension (`LuauFromNumber _) ->
      (not (word_type (Link.type_ types input))) || among_words bounds (Link.type_ types input) input.node.inputs.(0)
  | _ -> false

(* Luau: undiagnosed; converge or justify. *)
let written_words table write =
  let number = write.inputs.(2).node in
  let alone reader node = Readers.only_by table table.region reader node in
  match (number.kind, number.inputs) with
  | Extension `LuauToNumber, [| { node = { kind = Extension `LuauBitsIntoI64; _ } as words; _ } |]
    when alone write number && alone number words ->
      Some words
  | _ -> None

(** Luau's buffers are little-endian, so the low word goes first. *)
let write_words write words =
  let offset = write.inputs.(1) in
  let low = Numbers.extended `LuauBufferWriteU32 [| write.inputs.(0); offset; Numbers.to_number words.inputs.(0) |] in
  Numbers.extended `LuauBufferWriteU32
    [| low; Numbers.add offset (Numbers.constant 4.0); Numbers.to_number words.inputs.(1) |]

let written bounds low write =
  Option.map
    (fun value -> copied write [| write.inputs.(0); write.inputs.(1); value |])
    (* Luau: undiagnosed; converge or justify. *)
    (narrowed bounds low write.inputs.(2))

let written_f32 write =
  match viewed write.inputs.(2) with
  | { node = { kind = Extension `LuauBitsFromF32; inputs = [| number |]; _ }; _ } ->
      Some (Numbers.extended `LuauBufferWriteF32 [| write.inputs.(0); write.inputs.(1); number |])
  | _ -> None

let written_word bounds write =
  match written_f32 write with Some _ as direct -> direct | None -> written bounds ones write

(** Whether a read's bits, past views and the fences only waiting for it, go only to readers [wanted] accepts. *)
let rec read_by table wanted value =
  List.for_all
    (fun (reader, input) ->
      match reader.kind with
      | Fence -> input > 0
      | Extension (`LuauToNumber | `LuauFromNumber _) -> read_by table wanted (Link.output reader 0)
      | _ -> wanted reader)
    (Readers.of_link table table.region value)

let is_into_f32 reader = match reader.kind with Extension `LuauBitsIntoF32 -> true | _ -> false

let is_split reader =
  match reader.kind with
  | Extension (`LuauBitsFromI64 | `LuauBitsLowFromI64 | `LuauBitsHighFromI64) -> true
  | _ -> false

let extends_sign table amount reader =
  is_left_shift amount reader && read_by table (is_right_shift amount) (Link.output reader 0)

(** Each form gives the read's bits as its only readers take them, so what a fence waits for stays read. *)
let read_as_wanted table read =
  let wanted test = read_by table test (Link.output read 0) in
  let low signed mask =
    Numbers.to_number (Numbers.extended `LuauBit32Band [| Numbers.extended signed read.inputs; Numbers.constant mask |])
  in
  let word offset = Numbers.of_number `I32 (Numbers.extended `LuauBufferReadU32 [| read.inputs.(0); offset |]) in
  match read.kind with
  | Extension `LuauBufferReadU8 when wanted (extends_sign table 24) -> Some (low `LuauBufferReadI8 255.0)
  | Extension `LuauBufferReadU16 when wanted (extends_sign table 16) -> Some (low `LuauBufferReadI16 65535.0)
  | Extension `LuauBufferReadU32 when wanted is_into_f32 ->
      Some (Numbers.to_number (Numbers.extended `LuauBitsFromF32 [| Numbers.extended `LuauBufferReadF32 read.inputs |]))
  | Extension `LuauBufferReadF64 when wanted is_split ->
      let high = word (Numbers.add read.inputs.(1) (Numbers.constant 4.0)) in
      Some (Numbers.to_number (Numbers.extended `LuauBitsIntoI64 [| word read.inputs.(1); high |]))
  | _ -> None

(** A number just read as an F32 already holds what rounding gives. *)
let into_f32 into =
  match viewed into.inputs.(0) with
  | { node = { kind = Extension `LuauBitsFromF32; inputs = [| number |]; _ }; _ } -> (
      match number.node.kind with
      | Extension `LuauBufferReadF32 -> Some number
      | _ -> Some (Numbers.extended `LuauBitsRoundF32 [| number |]))
  | _ -> None

let one_word table split =
  let read port = Readers.is_read table table.region (Link.output split port) in
  match (read 0, read 1) with
  | true, false -> Some [| Numbers.extended `LuauBitsLowFromI64 split.inputs; Numbers.integer 0 |]
  | false, true -> Some [| Numbers.integer 0; Numbers.extended `LuauBitsHighFromI64 split.inputs |]
  | _ -> None

let joined_word port split =
  match split.inputs.(0).node with { kind = Extension `LuauBitsIntoI64; inputs; _ } -> Some inputs.(port) | _ -> None

(** An I64 joined from one word and zero rounds once, from the word's exact number. *)
let converted_word conversion =
  match conversion.inputs.(0).node with
  | { kind = Extension `LuauBitsIntoI64; inputs = [| low; high |]; _ } when is_zero high ->
      Some (Numbers.extended `LuauBitsFromF32 [| Numbers.to_number low |])
  | _ -> None

(** So does one joined from a word and its sign, as a signed value. *)
let converted_signed_word conversion =
  match conversion.inputs.(0).node with
  | { kind = Extension `LuauBitsIntoI64; inputs = [| low; high |]; _ }
    when is_right_shift 31 (viewed high).node && Link.equal (viewed (viewed high).node.inputs.(0)) (viewed low) ->
      Some (Numbers.extended `LuauBitsFromF32 [| Numbers.signed low |])
  | _ -> converted_word conversion

let bit_operations =
  (* Luau: undiagnosed; converge or justify. *)
  Options.optimization "simplify-luau-bit32" Integer Identities 1 "Remove Luau bit32 operations by known bits."

let views = Options.optimization "collapse-luau-views" Conversion Identities 1 "Collapse a Luau number viewed back."

let ranges =
  Options.optimization "simplify-luau-numbers" Number Identities 1 "Simplify Luau arithmetic by integer ranges."

let words =
  (* Luau: undiagnosed; converge or justify. *)
  Options.optimization "join-luau-words" Conversion Identities 1
    "Take an I64's words without splitting or joining more than is read."

let writes =
  (* Luau: undiagnosed; converge or justify. *)
  Options.optimization "narrow-luau-writes" Storage Narrowing 1 "Write only the bits a Luau buffer write keeps."

let reads =
  Options.optimization "narrow-luau-reads" Storage Narrowing 1 "Read a Luau buffer as the view its readers take."

let simplified options table bounds node =
  let applied = Options.applied options in
  let on operand_type = Link.type_ (LuauBounds.types bounds) node.inputs.(0) = operand_type in
  match node.kind with
  | Extension
      ( `LuauBit32Band | `LuauBit32Bor | `LuauBit32Bxor | `LuauBit32Lshift | `LuauBit32Rshift | `LuauBit32Arshift
      | `LuauBit32Lrotate | `LuauBit32Rrotate ) ->
      applied bit_operations (fun () -> bitwise bounds node)
  (* Luau: undiagnosed; converge or justify. *)
  | Extension (`LuauBit32Countlz | `LuauBit32Countrz) ->
      applied bit_operations (fun () -> replaced (narrowed bounds ones) node)
  | Extension `LuauWrap -> applied bit_operations (fun () -> unwrapped bounds node)
  | Extension (`LuauFromNumber _ | `LuauToNumber) when options views && viewed_back bounds node node.inputs.(0) ->
      Some node.inputs.(0).node.inputs.(0)
  (* Luau: undiagnosed; converge or justify. *)
  | Extension (`LuauFloorDivide | `LuauModulo | `LuauMathFmod) -> applied ranges (fun () -> divided bounds node)
  | Extension `LuauBitsIntoI64 -> applied words (fun () -> repacked node.inputs.(0) node.inputs.(1))
  | Extension `LuauBufferWriteU8 -> applied writes (fun () -> written bounds 0xFF node)
  | Extension `LuauBufferWriteU16 -> applied writes (fun () -> written bounds 0xFFFF node)
  (* Luau: undiagnosed; converge or justify. *)
  | Extension `LuauBufferWriteU32 -> applied writes (fun () -> written_word bounds node)
  | Extension (`LuauBufferReadU8 | `LuauBufferReadU16 | `LuauBufferReadU32 | `LuauBufferReadF64) ->
      applied reads (fun () -> read_as_wanted table node)
  | Extension `LuauBitsIntoF32 -> applied views (fun () -> into_f32 node)
  | Operator (Conversion (ConvertSigned `F32)) -> applied words (fun () -> converted_signed_word node)
  | Operator (Conversion (ConvertUnsigned `F32)) -> applied words (fun () -> converted_word node)
  | Extension `LuauBitsLowFromI64 -> applied words (fun () -> joined_word 0 node)
  | Extension `LuauBitsHighFromI64 -> applied words (fun () -> joined_word 1 node)
  | Extension `LuauBufferWriteF64 -> applied writes (fun () -> Option.map (write_words node) (written_words table node))
  | Extension `LuauAdd -> applied ranges (fun () -> arithmetic bounds node Add)
  | Extension `LuauSubtract -> applied ranges (fun () -> arithmetic bounds node Subtract)
  | Extension `LuauMultiply -> applied ranges (fun () -> arithmetic bounds node Multiply)
  | Extension (`LuauMathFloor | `LuauMathCeil | `LuauMathModf) -> applied ranges (fun () -> rounded bounds node)
  | Extension `LuauEqual when on Type.f64 -> applied ranges (fun () -> compared bounds node Equal)
  | Extension `LuauLessThan -> applied ranges (fun () -> compared bounds node LessThan)
  | Extension `LuauLessThanEqual -> applied ranges (fun () -> compared bounds node LessThanEqual)
  | _ -> None

(* Luau: undiagnosed; converge or justify. *)
let fold options types table =
  let bounds = LuauBounds.create types in
  fun node ->
    match (node.kind, node.inputs) with
    | Extension `LuauBitsFromI64, [| { node = { kind = Extension `LuauBitsIntoI64; _ } as joined; _ } |]
      when options words ->
        Some (Node.inputs joined)
    | Extension `LuauBitsFromI64, _ when options words -> one_word table node
    | _ -> Option.map (fun simplified -> [| simplified |]) (simplified options table bounds node)
