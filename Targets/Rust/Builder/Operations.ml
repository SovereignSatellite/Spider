open IR
open Definitions
open Allocator
open Emission
open Tree
open RustExtension

type reader = (x, Expression.t) Computation.reader
type computation = (Expression.t, Statement.t) Computation.t

let call path arguments : Expression.t = Call (Name path, arguments)

let call_helper (context : Context.t) name arguments =
  RuntimeLibrary.NeededHelpers.need context.helpers name;
  call (Helper name) arguments

(** The type is spelled out: Rust cannot infer it where the call is inlined into a cast or method call. *)
let call_generic (context : Context.t) name generics arguments : Expression.t =
  RuntimeLibrary.NeededHelpers.need context.helpers name;
  Call (Generic (Helper name, generics), arguments)

let each_leaf (value : int -> Expression.t) = Computation.each_leaf value
let single (value : Expression.t) = Computation.single value
let computed_where_read _ = true
let leaf_values (context : Context.t) reader link = Computation.leaf_values context.layouts reader link
let value (reader : reader) link = reader.value link 0
let types (context : Context.t) = Layouts.types context.layouts
let output_type (context : Context.t) node = RustTypes.leaf_type context.rust (Node.output_type (types context) node 0)

module Spelling = struct
  type expression = Expression.t

  let truth truth : expression = Name (Text (if truth then "true" else "false"))
  let equal value variant : expression = Binary (Equal, value, Integer (Int64.of_int variant, ""))
  let both first second : expression = Binary (And, first, second)
  let either first second : expression = Binary (Or, first, second)
  let if_else condition value otherwise : expression = IfElse (condition, value, otherwise)
  let not_ condition : expression = Unary (Not, condition)
end

module Conditions = Conditions.Make (Spelling)

let element (context : Context.t) (reader : reader) array leaf index : Expression.t =
  let leaves = Layouts.leaves_of context.layouts (Link.type_ (types context) array) in
  Method
    (Cast (reader.value array leaf, RustTypes.element_pointer context.rust leaves.(leaf)), "add", [ value reader index ])

let slot reference leaf : Expression.t = Field (Unary (Dereference, reference), RustTypes.leaf_field leaf)
let each_array (context : Context.t) array statement = Computation.each_array context.layouts array statement

let element_value (context : Context.t) reader element leaf =
  Computation.element_value context.layouts (Expression.Tuple []) reader element leaf

let unsigned_suffix bits = match bits with 8 -> "u8" | 16 -> "u16" | 32 -> "u32" | _ -> "u64"
let from_bits number : Name.t = Text (if number = Type.f32 then "f32::from_bits" else "f64::from_bits")

let create_constant context constant bits =
  let result_type = Node.output_type (types context) constant 0 in
  let integer = Expression.Integer (bits, unsigned_suffix (Type.bits result_type)) in
  match Type.scalar result_type with Some #Scalar.number -> call (from_bits result_type) [ integer ] | _ -> integer

(** Rust's [min] and [max] ignore NaN and may return either zero, and its [minimum] and [maximum] are unstable. *)
let operator node operand =
  let binary operator =
    let left = operand 0 in
    Some (Expression.Binary (operator, left, operand 1))
  in
  let method_ name =
    let receiver = operand 0 in
    Some (Expression.Method (receiver, name, [ operand 1 ]))
  in
  let unary name = Some (Expression.Method (operand 0, name, [])) in
  match node.kind with
  | Extension `RustWrappingAdd -> method_ "wrapping_add"
  | Extension `RustWrappingSub -> method_ "wrapping_sub"
  | Extension `RustWrappingMul -> method_ "wrapping_mul"
  | Extension `RustBitAnd -> binary BitwiseAnd
  | Extension `RustBitOr -> binary BitwiseOr
  | Extension `RustBitXor -> binary BitwiseExclusiveOr
  | Extension `RustAdd -> binary Add
  | Extension `RustSubtract -> binary Subtract
  | Extension `RustMultiply -> binary Multiply
  | Extension `RustDivide -> binary Divide
  | Extension `RustRemainder -> binary Remainder
  | Extension `RustNegate -> Some (Unary (Negate, operand 0))
  | Extension `RustCopysign -> method_ "copysign"
  | Extension `RustAbs -> unary "abs"
  | Extension `RustSqrt -> unary "sqrt"
  | Extension `RustCeil -> unary "ceil"
  | Extension `RustFloor -> unary "floor"
  | Extension `RustTrunc -> unary "trunc"
  | Extension `RustRoundTiesEven -> unary "round_ties_even"
  | Extension `RustLeadingZeros -> unary "leading_zeros"
  | Extension `RustTrailingZeros -> unary "trailing_zeros"
  | Extension `RustCountOnes -> unary "count_ones"
  | Extension `RustWrappingRem -> method_ "wrapping_rem"
  | Extension (`RustWrappingShl _) -> method_ "wrapping_shl"
  | Extension (`RustWrappingShr _ | `RustSignedWrappingShr _) -> method_ "wrapping_shr"
  | Extension (`RustRotateLeft _) -> method_ "rotate_left"
  | Extension (`RustRotateRight _) -> method_ "rotate_right"
  | _ -> None

(** [!=] negates [==] exactly, but an ordering takes [!], since [!(a < b)] and [a >= b] differ where either is NaN. *)
let compare (operator : RustOperator.binary) reader node variant : Expression.t =
  let left = value reader node.inputs.(0) in
  let right = value reader node.inputs.(1) in
  match (operator, variant) with
  | Equal, 0 -> Binary (NotEqual, left, right)
  | _ -> Conditions.holds (Binary (operator, left, right)) variant

let create_sum (context : Context.t) sum_type variant =
  Computation.create_sum context.layouts
    (fun variant -> Expression.Integer (Int64.of_int variant, "u32"))
    sum_type variant

let applied (context : Context.t) reader node : Expression.t =
  let callee = value reader node.inputs.(0) in
  Call (callee, Computation.input_values context.layouts reader node 1)

let choose (context : Context.t) reader matcher =
  let many tag runs = Expression.Match (tag, runs) in
  single (Conditions.choose_value context.layouts many reader matcher)

let reference_function context body definition : Expression.t =
  let referenced = Context.function_of context body in
  Cast (referenced, output_type context definition)

let name text : Expression.t = Bytes text

(** The types are spelled out, as Rust infers no result where the call is inlined into a method call, and no payload
    holding a null pointer. *)
let throw (context : Context.t) reader node : computation =
  let payload = Computation.input_values context.layouts reader node 0 in
  let generics =
    List.map (RustTypes.result_type context.rust)
      [ Link.types (types context) node.inputs; Node.output_types (types context) node ]
  in
  All (call_generic context "Throw" generics [ Expression.grouped payload ])

let attempt context reader node : computation =
  All (call_helper context "RustTry" [ Closure (applied context reader node) ])

(** The payload's type is spelled out, as nothing infers it where the payload is discarded. *)
let caught context payload_types outcome : computation =
  All (call_generic context "RustCaught" [ RustTypes.result_type context.rust payload_types ] [ outcome ])

let create_array context element_type bytes =
  call_generic context "RustArrayOf" [ RustTypes.leaf_type context.rust element_type ] [ Bytes bytes ]

let new_box context reader node =
  let fields =
    List.mapi (fun leaf content -> (RustTypes.leaf_field leaf, content)) (leaf_values context reader node.inputs.(0))
  in
  call_helper context "RustBoxNew"
    [ Struct (RustTypes.box_name context.rust (Node.output_type (types context) node 0), fields) ]

let get_box_contents reader node = each_leaf (slot (value reader node.inputs.(0)))

let set_box_contents context reader node : computation =
  let reference = value reader node.inputs.(0) in
  Statements
    (List.mapi
       (fun leaf content -> Statement.Assign ([ slot reference leaf ], [ content ]))
       (leaf_values context reader node.inputs.(1)))

let cast context reader node =
  let operand = value reader node.inputs.(0) in
  single (Cast (operand, output_type context node))

let retype context reader node =
  let operand = value reader node.inputs.(0) in
  single (Transmute (operand, output_type context node))

let get_element context reader node =
  each_leaf (fun leaf -> Unary (Dereference, element context reader node.inputs.(0) leaf node.inputs.(1)))

let set_element context reader node =
  let array = node.inputs.(0) in
  each_array context
    (Link.type_ (types context) array)
    (fun leaf ->
      let target = Expression.Unary (Dereference, element context reader array leaf node.inputs.(1)) in
      Statement.Assign ([ target ], [ element_value context reader node.inputs.(2) leaf ]))

let copy_elements context reader node =
  let destination, source = (node.inputs.(0), node.inputs.(2)) in
  let count = value reader node.inputs.(4) in
  each_array context
    (Link.type_ (types context) destination)
    (fun leaf ->
      let from = element context reader source leaf node.inputs.(3) in
      let into = element context reader destination leaf node.inputs.(1) in
      Statement.Call (call (Text "core::ptr::copy") [ from; into; count ]))

let fill_elements context reader node =
  let array = node.inputs.(0) in
  let count = value reader node.inputs.(2) in
  each_array context
    (Link.type_ (types context) array)
    (fun leaf ->
      let start = element context reader array leaf node.inputs.(1) in
      let range = call (Text "core::slice::from_raw_parts_mut") [ start; count ] in
      Statement.Call (Method (range, "fill", [ element_value context reader node.inputs.(3) leaf ])))

let new_array context reader node =
  each_leaf (fun leaf ->
      let length = value reader node.inputs.(0) in
      call_helper context "RustArrayNew" [ length; element_value context reader node.inputs.(1) leaf ])

let allocated context reader node variant =
  match
    List.map (fun leaf -> Spelling.not_ (Method (leaf, "is_null", []))) (leaf_values context reader node.inputs.(0))
  with
  | first :: rest -> Conditions.holds (List.fold_left Spelling.both first rest) variant
  | [] -> invalid_arg "An array has a leaf."

(** [core::ptr::eq] stands for [==], which on slice pointers draws rustc's [ambiguous_wide_pointer_comparisons]. *)
let pointers_equal reader node variant =
  let left = value reader node.inputs.(0) in
  Conditions.holds (call (Text "core::ptr::eq") [ left; value reader node.inputs.(1) ]) variant

let test context reader node variant =
  match node.kind with
  | Extension `RustEqual -> Some (compare Equal reader node variant)
  | Extension `RustLessThan -> Some (compare LessThan reader node variant)
  | Extension `RustLessThanEqual -> Some (compare LessThanEqual reader node variant)
  | Extension `RustPointerEqual -> Some (pointers_equal reader node variant)
  | Extension (`RustAllocated _) -> Some (allocated context reader node variant)
  | Extension `RustThrew -> Some (Conditions.holds (Method (value reader node.inputs.(0), "is_err", [])) variant)
  | _ -> None

let select tag runs : Conditions.condition =
  Holds (Match (tag, List.map (fun (variants, arm) -> (variants, Conditions.render arm)) runs))

(** Rust infers a call's results only from what it reads, so a call reading nothing spells them. *)
let called context reader node =
  match test context reader node 1 with
  | Some condition -> single (BooleanToInteger condition)
  | None ->
      let name_of = RuntimeLibrary.helper_name (types context) node in
      let call =
        if Array.length node.inputs = 0 then
          call_generic context name_of [ RustTypes.result_type context.rust (Node.output_types (types context) node) ]
        else call_helper context name_of
      in
      Computation.calls context.layouts (Expression.Tuple []) name call (fun call -> Statement.Call call) reader node

let at context reader node value_type : Expression.t =
  Cast (element context reader node.inputs.(0) 0 node.inputs.(1), Pointer (RustTypes.leaf_type context.rust value_type))

let read context reader node value_type = single (Method (at context reader node value_type, "read_unaligned", []))

let write context reader node value_type : computation =
  Statements [ Call (Method (at context reader node value_type, "write_unaligned", [ value reader node.inputs.(2) ])) ]

let compute context reader node =
  let operand index = value reader node.inputs.(index) in
  match (node.kind, Node.bits node.kind) with
  | _, Some bits -> single (create_constant context node bits)
  | Extension (`RustCast _), _ -> cast context reader node
  | Extension (`RustFromBits target), _ -> single (call (from_bits target) [ operand 0 ])
  | Extension (`RustToBits _), _ -> single (Method (operand 0, "to_bits", []))
  | SumNew { sum_type; variant }, _ -> create_sum context sum_type variant
  | RegionMatch _, _ -> choose context reader node
  | RegionReference body, _ -> single (reference_function context body node)
  | FunctionApply, _ -> All (applied context reader node)
  | Throw _, _ -> throw context reader node
  | Extension (`RustArrayOf (_, element_type, bytes)), _ -> single (create_array context element_type bytes)
  | Extension (`RustBoxNew _), _ -> single (new_box context reader node)
  | Extension (`RustBoxGet _), _ -> get_box_contents reader node
  | Extension (`RustBoxSet _), _ -> set_box_contents context reader node
  | Extension (`RustRetype _), _ -> retype context reader node
  | Extension (`RustElementGet _), _ -> get_element context reader node
  | Extension (`RustElementSet _), _ -> set_element context reader node
  | Extension (`RustElementCopy _), _ -> copy_elements context reader node
  | Extension (`RustElementFill _), _ -> fill_elements context reader node
  | Extension (`RustLength _), _ -> single (Method (operand 0, "len", []))
  | Extension (`RustRead (_, _, value_type)), _ -> read context reader node value_type
  | Extension (`RustWrite (_, _, value_type)), _ -> write context reader node value_type
  | Extension (`RustOutOfBounds _), _ -> single (call_helper context "RustOutOfBounds" [])
  | Extension (`RustArrayNew _), _ -> new_array context reader node
  | Extension `RustTry, _ -> attempt context reader node
  | Extension `RustReturned, _ -> All (Method (operand 0, "unwrap_unchecked", []))
  | Extension (`RustCaught payload_types), _ -> caught context payload_types (operand 0)
  | _ -> ( match operator node operand with Some expression -> single expression | None -> called context reader node)
