open IR
open Definitions
open Passes
open LuaJITExtension
open LuaJITValues

let is_zero link = modular_constant link = Some 0
let is_positive_zero link = Constant.bits link = Some 0L
let ranged bounds link = Option.is_some (LuaJITBounds.range bounds link)

let non_negative bounds link =
  match LuaJITBounds.range bounds link with Some range -> range.least >= 0.0 | None -> false

let copied node inputs = Link.output (Copying.node node inputs) 0

let as_word bounds value =
  match (value.node.kind, LuaJITBounds.range bounds value) with
  | Extension `LuaJITToNumber, _ when Link.type_ (LuaJITBounds.types bounds) value.node.inputs.(0) = Type.i32 ->
      Some value.node.inputs.(0)
  (* LuaJIT: undiagnosed; converge or justify. *)
  | _, Some range when range.least >= -2147483648.0 && range.greatest < 2147483648.0 ->
      Some (Numbers.of_number `I32 value)
  | _ -> None

let commutative node =
  match node.kind with Extension (`LuaJITBitBand | `LuaJITBitBor | `LuaJITBitBxor) -> true | _ -> false

let with_constant node =
  match (modular_constant node.inputs.(1), modular_constant node.inputs.(0)) with
  | Some constant, _ -> Some (node.inputs.(0), constant)
  | None, Some constant when commutative node -> Some (node.inputs.(1), constant)
  | _ -> None

(* LuaJIT: undiagnosed; converge or justify. *)

(** [bit.tobit] also takes an I64's low 32 bits, which no number operand can replace. *)
let keeping types low link =
  let node = link.node in
  match (node.kind, if commutative node then with_constant node else None) with
  | Extension `LuaJITBitBand, Some (operand, constant) when constant land low = low -> Some operand
  | Extension (`LuaJITBitBor | `LuaJITBitBxor), Some (operand, constant) when constant land low = 0 -> Some operand
  (* LuaJIT: undiagnosed; converge or justify. *)
  | Extension `LuaJITBitTobit, _ when Node.operand_type types node = Type.f64 -> Some node.inputs.(0)
  | _ -> None

let wrap bounds value = match as_word bounds value with Some _ as word -> word | None -> Some (Numbers.wrap value)

(* LuaJIT: undiagnosed; converge or justify. *)

(** [bit] takes a shift or rotation amount modulo 32, so shifts fuse only below it. *)
let fused_constant node inner outer =
  let sum = (inner land 31) + (outer land 31) in
  match node.kind with
  | Extension `LuaJITBitBand -> Some (inner land outer)
  | Extension `LuaJITBitBor -> Some (inner lor outer)
  | Extension `LuaJITBitBxor -> Some (inner lxor outer)
  (* LuaJIT: undiagnosed; converge or justify. *)
  | Extension (`LuaJITBitLshift | `LuaJITBitRshift) -> if sum < 32 then Some sum else None
  | Extension `LuaJITBitArshift -> Some (min sum 31)
  | Extension (`LuaJITBitRol | `LuaJITBitRor) -> Some (sum mod 32)
  | _ -> None

let fused_binary types node inner outer =
  match (with_constant inner, node.kind, inner.kind) with
  | Some (source, constant), _, _ when Node.same_kind types inner node ->
      Option.map
        (fun fused -> copied node [| source; Numbers.constant (float_of_int fused) |])
        (fused_constant node constant outer)
  | Some (source, constant), Extension `LuaJITBitBand, Extension (`LuaJITBitBor | `LuaJITBitBxor)
    when constant land outer = 0 ->
      Some (copied node [| source; Numbers.constant (float_of_int outer) |])
  | _ -> None

let fused_operand types node operand outer =
  let inner = operand.node.inputs.(0).node in
  match inner.kind with
  | Extension
      ( `LuaJITBitBand | `LuaJITBitBor | `LuaJITBitBxor | `LuaJITBitLshift | `LuaJITBitRshift | `LuaJITBitArshift
      | `LuaJITBitRol | `LuaJITBitRor ) ->
      fused_binary types node inner outer
  | _ -> None

let fused types node =
  match with_constant node with
  | Some (({ node = { kind = Extension `LuaJITToNumber; _ }; _ } as operand), outer) ->
      fused_operand types node operand outer
  | _ -> None

let replaced replacement node =
  let inputs = node.inputs in
  let replacements = Array.map replacement inputs in
  if Array.for_all Option.is_none replacements then None
  else
    Some
      (copied node (Array.map2 (fun input replacement -> Option.value replacement ~default:input) inputs replacements))

(* LuaJIT: undiagnosed; converge or justify. *)

(** A left shift reads its operand modulo 2^32, so modulo 2^32 the negation of its result is the shift of its operand's
    negation. *)
let negated_shift operand =
  match (operand.node.kind, operand.node.inputs) with
  | ( Extension `LuaJITSubtract,
      [| zero; { node = { kind = Extension `LuaJITToNumber; inputs = [| { node = shift; _ } |]; _ }; _ } |] )
    when number zero = Some 0.0 -> (
      match (shift.kind, shift.inputs) with
      | Extension `LuaJITBitLshift, [| shifted; amount |] ->
          Some (Numbers.to_number (copied shift [| Numbers.subtract zero shifted; amount |]))
      | _ -> None)
  | _ -> None

(* LuaJIT: undiagnosed; converge or justify. *)

(** A consumer taking its operand modulo 2^width, as [bit] and FFI's integer writes do, needs only its [low] bits, so an
    inner [bit] operation keeping them can go, also from a term of an exact sum or difference. *)
let narrowed bounds low operand =
  let term link =
    match link.node.kind with
    (* LuaJIT: undiagnosed; converge or justify. *)
    | Extension `LuaJITToNumber -> keeping (LuaJITBounds.types bounds) low link.node.inputs.(0)
    | _ -> None
  in
  match operand.node.kind with
  | Extension `LuaJITToNumber -> term operand
  | Extension (`LuaJITAdd | `LuaJITSubtract) -> (
      match (negated_shift operand, replaced term operand.node) with
      | (Some _ as shift), _ -> shift
      | None, Some sum when ranged bounds operand && ranged bounds sum -> Some sum
      | _ -> None)
  | _ -> None

let is_ones link = modular_constant link = Some ones

let reduced bounds node =
  let left, right = (node.inputs.(0), node.inputs.(1)) in
  let left_ones = LuaJITBounds.operand_ones bounds left in
  let right_ones = LuaJITBounds.operand_ones bounds right in
  let is_constant link = Option.is_some (modular_constant link) in
  match node.kind with
  | _ when LuaJITBounds.ones bounds (Link.output node 0) = 0 -> Some (Numbers.integer 0)
  | Extension `LuaJITBitBxor when Link.equal left right -> Some (Numbers.integer 0)
  | Extension (`LuaJITBitBand | `LuaJITBitBor) when Link.equal left right -> wrap bounds left
  | Extension `LuaJITBitBand when left_ones land lnot right_ones = 0 && is_constant right -> wrap bounds left
  | Extension `LuaJITBitBand when right_ones land lnot left_ones = 0 && is_constant left -> wrap bounds right
  | Extension (`LuaJITBitBor | `LuaJITBitBxor) when right_ones = 0 -> wrap bounds left
  | Extension (`LuaJITBitBor | `LuaJITBitBxor) when left_ones = 0 -> wrap bounds right
  | Extension `LuaJITBitBor when left_ones land lnot right_ones = 0 && is_constant right ->
      Some (Numbers.integer right_ones)
  | Extension `LuaJITBitBor when right_ones land lnot left_ones = 0 && is_constant left ->
      Some (Numbers.integer left_ones)
  | Extension (`LuaJITBitBand | `LuaJITBitBor | `LuaJITBitBxor) -> None
  | _ when is_zero right -> wrap bounds left
  | Extension (`LuaJITBitArshift | `LuaJITBitRol | `LuaJITBitRor) when is_ones left -> Some (Numbers.integer ones)
  | _ -> None

(* LuaJIT: undiagnosed; converge or justify. *)

(** A [bit] operation with a constant whose bits the operand lacks is their sum, which is the I32's signed value only
    below 2^31. *)
let added bounds node =
  match (node.kind, with_constant node) with
  | Extension (`LuaJITBitBor | `LuaJITBitBxor), Some (operand, constant) when constant <> 0 -> (
      match LuaJITBounds.word_ones bounds operand with
      (* LuaJIT: undiagnosed; converge or justify. *)
      | Some ones when ones land constant = 0 && ones lor constant < 0x8000_0000 ->
          Some (Numbers.of_number `I32 (Numbers.add operand (Numbers.constant (float_of_int constant))))
      | _ -> None)
  | _ -> None

let shift_of link =
  match (link.node.kind, link.node.inputs) with
  | ( Extension `LuaJITToNumber,
      [|
        {
          node =
            {
              kind = Extension (`LuaJITBitLshift | `LuaJITBitRshift | `LuaJITBitArshift | `LuaJITBitRol | `LuaJITBitRor);
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

let bitwise bounds node =
  List.find_map
    (fun rule -> rule node)
    [
      reduced bounds;
      added bounds;
      fused (LuaJITBounds.types bounds);
      distributed (LuaJITBounds.types bounds);
      replaced (narrowed bounds ones);
    ]

let unwrapped bounds node =
  match as_word bounds node.inputs.(0) with Some _ as word -> word | None -> replaced (narrowed bounds ones) node

let interval bounds link =
  match (LuaJITBounds.range bounds link, number link) with
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
  | Extension ((`LuaJITAdd | `LuaJITSubtract) as operation) -> (
      match (operation, number inputs.(0), number inputs.(1)) with
      | `LuaJITAdd, _, Some constant -> Some { source = inputs.(0); negated = false; constant }
      | `LuaJITAdd, Some constant, _ -> Some { source = inputs.(1); negated = false; constant }
      | `LuaJITSubtract, _, Some constant -> Some { source = inputs.(0); negated = false; constant = 0.0 -. constant }
      | `LuaJITSubtract, Some constant, _ -> Some { source = inputs.(1); negated = true; constant }
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
  | Extension `LuaJITSubtract, [| zero; source |] when is_positive_zero zero -> Some source
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
      List.find_map (fun rule -> rule node) [ reassociated bounds; negation_carried operator ]
  | _ -> None

let rounded bounds node =
  let operand = node.inputs.(0) in
  (* LuaJIT: undiagnosed; converge or justify. *)
  if ranged bounds operand then Some operand else None

let divided bounds node =
  let left, right = (node.inputs.(0), node.inputs.(1)) in
  let below =
    match (LuaJITBounds.range bounds left, LuaJITBounds.range bounds right) with
    | Some dividend, Some divisor -> dividend.least >= 0.0 && dividend.greatest < divisor.least
    | _ -> false
  in
  let by_one = number right = Some 1.0 && ranged bounds left in
  match node.kind with
  | Extension (`LuaJITModulo | `LuaJITMathFmod) when below -> Some left
  | Extension `LuaJITModulo when by_one -> Some (Numbers.constant 0.0)
  | Extension `LuaJITMathFmod when by_one && non_negative bounds left -> Some (Numbers.constant 0.0)
  | _ -> None

(* LuaJIT: undiagnosed; converge or justify. *)
let written_number as_number write =
  match write.inputs.(3).node.kind with
  | Extension `LuaJITBitsFromF64 ->
      Some
        (Numbers.extended as_number
           [| write.inputs.(0); write.inputs.(1); write.inputs.(2); write.inputs.(3).node.inputs.(0) |])
  | _ -> None

(** An integer's number truncated to an I64 converts back to itself, and so do its low 32 bits as their unsigned value.
*)
let converted bounds long =
  let truncated link =
    match link.node.kind with
    | Extension (`LuaJITCast _) when ranged bounds link.node.inputs.(0) -> Some link.node.inputs.(0)
    | _ -> None
  in
  match long.node.kind with
  | Extension `LuaJITBitBand when Constant.bits long.node.inputs.(1) = Some 0xFFFF_FFFFL ->
      Option.map
        (fun number -> Numbers.extended `LuaJITModulo [| number; Numbers.constant 4294967296.0 |])
        (truncated long.node.inputs.(0))
  | _ -> truncated long

let among_words bounds word number =
  (* LuaJIT: undiagnosed; converge or justify. *)
  match (LuaJITBounds.range bounds number, word = Type.i32) with
  | Some range, true -> range.least >= -2147483648.0 && range.greatest < 2147483648.0
  | Some range, false -> range.least >= 0.0 && range.greatest <= Int64.to_float (Constant.mask word)
  | None, _ -> false

(** A word's number converted back collapses only where its range already fits the word, since otherwise the conversion
    is what gives the bounds that range. *)
let viewed_back bounds node input =
  let types = LuaJITBounds.types bounds in
  match (node.kind, input.node.kind) with
  | Extension (`LuaJITFromNumber _), Extension `LuaJITToNumber ->
      Node.operand_type types input.node = Node.output_type types node 0
  | Extension `LuaJITToNumber, Extension (`LuaJITFromNumber _) ->
      (not (word_type (Link.type_ types input))) || among_words bounds (Link.type_ types input) input.node.inputs.(0)
  | _ -> false

let written bounds low write =
  Option.map
    (* LuaJIT: undiagnosed; converge or justify. *)
    (fun value -> copied write [| write.inputs.(0); write.inputs.(1); write.inputs.(2); value |])
    (narrowed bounds low write.inputs.(3))

let bit_operations =
  (* LuaJIT: undiagnosed; converge or justify. *)
  Options.optimization "simplify-luajit-bit" Integer Identities 1 "Remove LuaJIT bit operations by known bits."

let views = Options.optimization "collapse-luajit-views" Conversion Identities 1 "Collapse a LuaJIT number viewed back."

let ranges =
  Options.optimization "simplify-luajit-numbers" Number Identities 1 "Simplify LuaJIT arithmetic by integer ranges."

let longs =
  (* LuaJIT: undiagnosed; converge or justify. *)
  Options.optimization "collapse-luajit-longs" Conversion Identities 1
    "Collapse an integer's number cast to an I64 and back."

let f64_bits =
  Options.optimization "unpack-luajit-f64-bits" Conversion Identities 1 "Write an F64 as one, not its bits."

let writes =
  (* LuaJIT: undiagnosed; converge or justify. *)
  Options.optimization "narrow-luajit-writes" Storage Narrowing 1 "Write only the bits a LuaJIT FFI write keeps."

let simplified options bounds node =
  let applied = Options.applied options in
  let on operand_type = Link.type_ (LuaJITBounds.types bounds) node.inputs.(0) = operand_type in
  match node.kind with
  | Extension
      ( `LuaJITBitBand | `LuaJITBitBor | `LuaJITBitBxor | `LuaJITBitLshift | `LuaJITBitRshift | `LuaJITBitArshift
      | `LuaJITBitRol | `LuaJITBitRor )
  (* LuaJIT: undiagnosed; converge or justify. *)
    when on Type.f64 ->
      applied bit_operations (fun () -> bitwise bounds node)
  | Extension `LuaJITBitTobit when on Type.f64 -> applied bit_operations (fun () -> unwrapped bounds node)
  | Extension (`LuaJITFromNumber _ | `LuaJITToNumber) when options views && viewed_back bounds node node.inputs.(0) ->
      Some node.inputs.(0).node.inputs.(0)
  (* LuaJIT: undiagnosed; converge or justify. *)
  | Extension (`LuaJITModulo | `LuaJITMathFmod) when on Type.f64 -> applied ranges (fun () -> divided bounds node)
  | Extension `LuaJITTonumber when on Type.i64 -> applied longs (fun () -> converted bounds node.inputs.(0))
  | Extension `LuaJITWriteU8 -> applied writes (fun () -> written bounds 0xFF node)
  | Extension `LuaJITWriteU16 -> applied writes (fun () -> written bounds 0xFFFF node)
  (* LuaJIT: undiagnosed; converge or justify. *)
  | Extension `LuaJITWriteI32 -> applied writes (fun () -> written bounds ones node)
  | Extension `LuaJITWriteI64 -> applied f64_bits (fun () -> written_number `LuaJITWriteF64 node)
  | Extension `LuaJITAdd when on Type.f64 -> applied ranges (fun () -> arithmetic bounds node Add)
  | Extension `LuaJITSubtract when on Type.f64 -> applied ranges (fun () -> arithmetic bounds node Subtract)
  | Extension `LuaJITMultiply when on Type.f64 -> applied ranges (fun () -> arithmetic bounds node Multiply)
  | Extension (`LuaJITMathFloor | `LuaJITMathCeil | `LuaJITMathModf) -> applied ranges (fun () -> rounded bounds node)
  | Extension `LuaJITEqual when on Type.f64 -> applied ranges (fun () -> compared bounds node Equal)
  (* LuaJIT: undiagnosed; converge or justify. *)
  | Extension `LuaJITLessThan when on Type.f64 -> applied ranges (fun () -> compared bounds node LessThan)
  | Extension `LuaJITLessThanEqual -> applied ranges (fun () -> compared bounds node LessThanEqual)
  | _ -> None

let fold options types =
  let bounds = LuaJITBounds.create types in
  fun node -> Option.map (fun simplified -> [| simplified |]) (simplified options bounds node)
