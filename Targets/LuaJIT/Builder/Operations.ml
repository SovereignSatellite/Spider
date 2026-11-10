open IR
open Definitions
open Allocator
open Emission
open Tree
open LuaJITExtension

type reader = (x, Expression.t) Computation.reader
type computation = (Expression.t, Statement.t) Computation.t

let call path arguments : Expression.t = Call (Name path, arguments)

let helper (context : Context.t) name : Expression.t =
  RuntimeLibrary.NeededHelpers.need context.helpers name;
  Name (Helper name)

let call_helper context name arguments : Expression.t = Call (helper context name, arguments)

(* LuaJIT: Lua has no if-expression, NaN tagging needs canonical NaNs, and FFI pointers access bytes. *)
let constant (context : Context.t) name : Expression.t =
  RuntimeLibrary.NeededHelpers.need context.helpers name;
  Name (Constant name)

let each_leaf value = Computation.each_leaf value
let single (value : Expression.t) = Computation.single value
let computed_where_read node = match node.kind with RegionMatch _ -> false | _ -> true
let leaf_values (context : Context.t) reader link = Computation.leaf_values context.layouts reader link
let value (reader : reader) link = reader.value link 0
let types (context : Context.t) = Layouts.types context.layouts

module Spelling = struct
  type expression = Expression.t

  let truth truth : expression = Name (Text (if truth then "true" else "false"))
  let equal value variant : expression = Binary (Equal, value, Integer (Int64.of_int variant))
  let both first second : expression = Binary (And, first, second)
  let either first second : expression = Binary (Or, first, second)

  (* LuaJIT: Lua has no if-expression, NaN tagging needs canonical NaNs, and FFI pointers access bytes. *)
  let if_else _ _ _ = invalid_arg "LuaJIT computes no Match where read."
  let not_ condition : expression = Unary (Not, condition)
end

module Conditions = Conditions.Make (Spelling)

let at_least tag lower upper variant : Expression.t =
  match (lower, upper) with
  | [ ([ only ], _) ], [ _ ] -> Binary ((if variant = 1 then NotEqual else Equal), tag, Integer (Int64.of_int only))
  | _ ->
      let bound = List.hd (fst (List.hd upper)) in
      Binary ((if variant = 1 then GreaterThanEqual else LessThan), tag, Integer (Int64.of_int bound))

let rec halved tag split runs =
  match runs with
  | [ (_, only) ] -> only
  | _ ->
      let half = List.length runs / 2 in
      let lower = List.take half runs in
      let upper = List.drop half runs in
      split (at_least tag lower upper) (halved tag split lower) (halved tag split upper)

let element (reader : reader) array leaf key : Expression.t =
  let operand = reader.value array leaf in
  Index (operand, value reader key)

let slot reference leaf : Expression.t = Index (reference, Integer (Int64.of_int (leaf + 1)))
let each_array (context : Context.t) array statement = Computation.each_array context.layouts array statement

let element_value (context : Context.t) reader element leaf =
  Computation.element_value context.layouts Expression.Nil reader element leaf

(* LuaJIT: Lua has no if-expression, NaN tagging needs canonical NaNs, and FFI pointers access bytes. *)

(** LuaJIT has no NaN literal, and [0/0] takes the machine's sign, so a NaN is made from the bits of LuaJIT's NaN of its
    sign. *)
let number_of_bits context value : Expression.t =
  let number = Int64.float_of_bits value in
  match Float.classify_float number with
  (* LuaJIT: Lua has no if-expression, NaN tagging needs canonical NaNs, and FFI pointers access bytes. *)
  | FP_normal | FP_subnormal | FP_zero -> Number value
  | FP_infinite ->
      let huge = helper context "MathHuge" in
      if number > 0.0 then huge else Unary (Negate, huge)
  | FP_nan ->
      (* LuaJIT: Lua has no if-expression, NaN tagging needs canonical NaNs, and FFI pointers access bytes. *)
      call_helper context "LuaJITBitsIntoF64"
        [ Long (Int64.logor (Int64.logand value Int64.min_int) 0x7FF8_0000_0000_0000L) ]

let literal context value_type bits : Expression.t =
  match Type.bits value_type with
  | 64 when value_type = Type.i64 -> Long bits
  | 64 -> number_of_bits context bits
  | 32 -> Integer (Int64.of_int32 (Int64.to_int32 bits))
  | _ -> Integer bits

let create_constant context constant bits = literal context (Node.output_type (types context) constant 0) bits

(** LuaJIT's [math.min] and [math.max] ignore a NaN first operand and do not order −0 below +0. *)
let operator node operand =
  let binary operator =
    let left = operand 0 in
    Some (Expression.Binary (operator, left, operand 1))
  in
  match node.kind with
  | Extension `LuaJITAdd -> binary Add
  | Extension `LuaJITSubtract -> binary Subtract
  | Extension `LuaJITMultiply -> binary Multiply
  | Extension `LuaJITDivide -> binary Divide
  | Extension `LuaJITNegate -> Some (Unary (Negate, operand 0))
  | Extension `LuaJITModulo -> binary Modulo
  | _ -> None

(** [~=] negates [==] exactly, but an ordering takes [not], since [not (a < b)] and [a >= b] differ where either is NaN.
*)
let compare (operator : LuaJITOperator.binary) reader node variant : Expression.t =
  let left = value reader node.inputs.(0) in
  let right = value reader node.inputs.(1) in
  match (operator, variant) with
  | Equal, 0 -> Binary (NotEqual, left, right)
  | _ -> Conditions.holds (Binary (operator, left, right)) variant

let create_sum (context : Context.t) sum_type variant =
  Computation.create_sum context.layouts (fun variant -> Expression.Integer (Int64.of_int variant)) sum_type variant

let applied (context : Context.t) reader node : Expression.t =
  let callee = value reader node.inputs.(0) in
  Call (callee, Computation.input_values context.layouts reader node 1)

let name text : Expression.t = Bytes text

let throw (context : Context.t) reader node : computation =
  let payload = Computation.input_values context.layouts reader node 0 in
  All (call_helper context "Throw" [ Expression.grouped payload ])

let caught (context : Context.t) payload_types thrown =
  let leaves = Layouts.leaves context.layouts payload_types in
  (* LuaJIT: Lua has no if-expression, NaN tagging needs canonical NaNs, and FFI pointers access bytes. *)
  each_leaf (if List.length leaves = 1 then fun _ -> thrown else slot thrown)

let literal_element context element_type bytes index =
  literal context element_type
    (match Type.bits element_type with
    | 16 -> Int64.of_int (String.get_uint16_le bytes (2 * index))
    | 32 -> Int64.of_int32 (String.get_int32_le bytes (4 * index))
    | _ -> String.get_int64_le bytes (8 * index))

let create_array context element_type bytes : Expression.t =
  let length = String.length bytes / (Type.bits element_type / 8) in
  let elements = List.init length (literal_element context element_type bytes) in
  Table (elements, [ (Text "length", Integer (Int64.of_int length)) ])

let new_box context reader node : Expression.t = Table (leaf_values context reader node.inputs.(0), [])

let get_box_contents reader node =
  let reference = value reader node.inputs.(0) in
  each_leaf (slot reference)

(* LuaJIT: Lua has no if-expression, NaN tagging needs canonical NaNs, and FFI pointers access bytes. *)

(** One assignment per slot: a single assignment of every slot would hold each target and value in a slot of its own,
    more than the 250 a LuaJIT function has for a large box. *)
let set_box_contents context reader node : computation =
  let reference = value reader node.inputs.(0) in
  Statements
    (List.mapi
       (fun leaf content -> Statement.Assign ([ slot reference leaf ], [ content ]))
       (leaf_values context reader node.inputs.(1)))

let get_element reader node = each_leaf (fun leaf -> element reader node.inputs.(0) leaf node.inputs.(1))

let set_element context reader node =
  let array = node.inputs.(0) in
  each_array context
    (Link.type_ (types context) array)
    (fun leaf ->
      let target = element reader array leaf node.inputs.(1) in
      Statement.Assign ([ target ], [ element_value context reader node.inputs.(2) leaf ]))

let move_table context reader node =
  let key index = value reader node.inputs.(index) in
  each_array context
    (Link.type_ (types context) node.inputs.(0))
    (fun leaf ->
      let table index = reader.value node.inputs.(index) leaf in
      Statement.Call (call (Text "table.move") [ table 2; key 3; key 4; key 1; table 0 ]))

let new_array context reader node =
  let length = value reader node.inputs.(0) in
  (* LuaJIT: Lua has no if-expression, NaN tagging needs canonical NaNs, and FFI pointers access bytes. *)
  each_leaf (fun _ -> call_helper context "LuaJITArrayNew" [ length ])

let allocated context reader node variant =
  match leaf_values context reader node.inputs.(0) with
  | first :: rest -> Conditions.holds (List.fold_left Spelling.both first rest) variant
  | [] -> invalid_arg "An array has a leaf."

let test context reader node variant =
  match node.kind with
  | Extension `LuaJITEqual -> Some (compare Equal reader node variant)
  | Extension `LuaJITLessThan -> Some (compare LessThan reader node variant)
  | Extension `LuaJITLessThanEqual -> Some (compare LessThanEqual reader node variant)
  | Extension (`LuaJITAllocated _) -> Some (allocated context reader node variant)
  | _ -> None

let select tag runs = halved tag (fun test -> Conditions.choose (fun variant -> Holds (test variant))) runs

let called context reader node =
  match test context reader node 1 with
  (* LuaJIT: Lua has no if-expression, NaN tagging needs canonical NaNs, and FFI pointers access bytes. *)
  | Some condition -> single (Binary (Or, Binary (And, condition, Integer 1L), Integer 0L))
  | None ->
      let call = call_helper context (RuntimeLibrary.helper_name (types context) node) in
      Computation.calls context.layouts Expression.Nil name call (fun call -> Statement.Call call) reader node

(* LuaJIT: Lua has no if-expression, NaN tagging needs canonical NaNs, and FFI pointers access bytes. *)
let data_at reader bytes offset : Expression.t =
  Binary (Add, Field (value reader bytes, Text "data"), value reader offset)

(** An access is through [pointer], the type it reads or writes, at the bytes' data plus the address and the offset; a
    write is followed by the barrier no access crosses. *)
let pointed context reader node pointer : Expression.t =
  let at : Expression.t = Binary (Add, data_at reader node.inputs.(0) node.inputs.(1), value reader node.inputs.(2)) in
  Index (call_helper context "FfiCast" [ helper context pointer; at ], Integer 0L)

let written context reader node pointer : computation =
  Statements
    [
      Assign ([ pointed context reader node pointer ], [ value reader node.inputs.(3) ]);
      Call (call_helper context "FfiFill" [ constant context "Barrier"; Integer 1L ]);
    ]

let fill_bytes context reader node : computation =
  let inputs = node.inputs in
  Statements
    [
      Call
        (call_helper context "FfiFill"
           [ data_at reader inputs.(0) inputs.(1); value reader inputs.(2); value reader inputs.(3) ]);
    ]

(** [memmove] copies overlapping ranges as if through a temporary, which [ffi.copy] does not. *)
let copy_bytes (context : Context.t) reader node : computation =
  let inputs = node.inputs in
  RuntimeLibrary.NeededHelpers.need context.helpers "Memmove";
  let memmove : Expression.t = Field (Field (helper context "Ffi", Text "C"), Text "memmove") in
  Statements
    [
      Call
        (Call
           ( memmove,
             [ data_at reader inputs.(0) inputs.(1); data_at reader inputs.(2) inputs.(3); value reader inputs.(4) ] ));
    ]

let cast context target operand =
  call_helper context "FfiCast" [ helper context (if target = Type.i64 then "Int64" else "Uint64"); operand ]

let compute context reader node =
  let operand index = value reader node.inputs.(index) in
  match (node.kind, Node.bits node.kind) with
  | _, Some bits -> single (create_constant context node bits)
  | SumNew { sum_type; variant }, _ -> create_sum context sum_type variant
  | RegionReference body, _ -> single (Context.function_of context body)
  | FunctionApply, _ -> All (applied context reader node)
  | Throw _, _ -> throw context reader node
  | Extension (`LuaJITArrayOf (_, element_type, bytes)), _ -> single (create_array context element_type bytes)
  | Extension (`LuaJITBytesFromString bytes), _ -> single (call_helper context "LuaJITBytesFromString" [ Bytes bytes ])
  | Extension (`LuaJITLength _), _ -> single (Field (operand 0, Text "length"))
  | Extension (`LuaJITBoxNew _), _ -> single (new_box context reader node)
  | Extension (`LuaJITBoxGet _), _ -> get_box_contents reader node
  | Extension (`LuaJITBoxSet _), _ -> set_box_contents context reader node
  (* LuaJIT: Lua has no if-expression, NaN tagging needs canonical NaNs, and FFI pointers access bytes. *)
  | Extension (`LuaJITRetype _), _ -> single (operand 0)
  | Extension (`LuaJITTableGet _), _ -> get_element reader node
  | Extension (`LuaJITTableSet _), _ -> set_element context reader node
  | Extension (`LuaJITTableMove _), _ -> move_table context reader node
  (* LuaJIT: Lua has no if-expression, NaN tagging needs canonical NaNs, and FFI pointers access bytes. *)
  | Extension `LuaJITReadU8, _ -> single (pointed context reader node "Uint8Pointer")
  | Extension `LuaJITReadU16, _ -> single (pointed context reader node "Uint16Pointer")
  | Extension `LuaJITReadI32, _ -> single (pointed context reader node "Int32Pointer")
  | Extension `LuaJITReadI64, _ -> single (pointed context reader node "Int64Pointer")
  | Extension `LuaJITReadF64, _ -> single (pointed context reader node "DoublePointer")
  | Extension `LuaJITWriteU8, _ -> written context reader node "Uint8Pointer"
  | Extension `LuaJITWriteU16, _ -> written context reader node "Uint16Pointer"
  | Extension `LuaJITWriteI32, _ -> written context reader node "Int32Pointer"
  | Extension `LuaJITWriteI64, _ -> written context reader node "Int64Pointer"
  | Extension `LuaJITWriteF64, _ -> written context reader node "DoublePointer"
  | Extension `LuaJITBytesFill, _ -> fill_bytes context reader node
  | Extension `LuaJITBytesCopy, _ -> copy_bytes context reader node
  | Extension (`LuaJITCast target), _ -> single (cast context target (operand 0))
  | Extension (`LuaJITArrayNew _), _ -> new_array context reader node
  | Extension (`LuaJITCaught payload_types), _ -> caught context payload_types (operand 0)
  | _ -> ( match operator node operand with Some expression -> single expression | None -> called context reader node)
