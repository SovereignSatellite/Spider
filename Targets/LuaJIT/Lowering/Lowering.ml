open IR
open Definitions
open Passes
open LuaJITExtension
open Numbers

let integer_binary types node width operator =
  let left, right = (node.inputs.(0), node.inputs.(1)) in
  match width with `I32 -> Integers.binary operator left right | `I64 -> Longs.binary types operator left right

let integer_count node width operator =
  let value = node.inputs.(0) in
  match width with `I32 -> Integers.count operator value | `I64 -> Longs.count operator value

let integer_comparison types node width operator =
  let left, right = (node.inputs.(0), node.inputs.(1)) in
  match width with `I32 -> Integers.compare operator left right | `I64 -> Longs.compare types operator left right

let sign_extend width from value =
  match width with
  | `I32 -> Integers.sign_extend (Scalar.bits from) value
  | `I64 -> Longs.sign_extend (Scalar.bits from) value

let narrow_i32 target value =
  retype
    (target :> Scalar.t)
    (extended `LuaJITBitBand [| to_number value; constant (Int64.to_float (Constant.mask (Type.of_scalar target))) |])

(* LuaJIT: I64s lower to FFI longs, NaN tagging reads other NaNs as other values, and FFI accesses are unchecked. *)
let widen types source value =
  match source with `F32 -> Floats.widen types value | `I32 -> Longs.widen value | `I8 | `I16 -> retype `I32 value

let narrow target value =
  match target with
  | `F32 -> Floats.narrow value
  | `I32 -> Longs.narrow value
  | (`I8 | `I16) as target -> narrow_i32 target value

(* LuaJIT: I64s lower to FFI longs, NaN tagging reads other NaNs as other values, and FFI accesses are unchecked. *)
let reinterpret types target value =
  match target with
  | `F64 -> Longs.into_f64 types value
  | `I64 -> extended `LuaJITBitsFromF64 [| value |]
  | (`I32 | `F32) as target -> retype target value

let conversion types node (conversion : Conversion.t) =
  let value = node.inputs.(0) in
  match conversion with
  (* LuaJIT: I64s lower to FFI longs, NaN tagging reads other NaNs as other values, and FFI accesses are unchecked. *)
  | Widen source -> Some (widen types source value)
  | Narrow target -> Some (narrow target value)
  | SignExtend (width, from) -> Some (sign_extend width from value)
  | ConvertSigned target -> Floats.convert_signed target value
  (* LuaJIT: I64s lower to FFI longs, NaN tagging reads other NaNs as other values, and FFI accesses are unchecked. *)
  | ConvertUnsigned target -> Floats.convert_unsigned types target value
  | SaturateSigned target -> Some (Floats.saturate_signed types target value)
  | SaturateUnsigned target -> Some (Floats.saturate_unsigned types target value)
  (* LuaJIT: I64s lower to FFI longs, NaN tagging reads other NaNs as other values, and FFI accesses are unchecked. *)
  | Retype target -> Some (reinterpret types target value)

let attempt types attempt payload_types =
  ExtensionLowering.attempt types attempt
    (fun inputs -> Node.outputs types (IR_LuaJIT.try_ inputs.(0) (Array.sub inputs 1 (Array.length inputs - 1))))
    Fun.id
    (fun fields -> Array.sub fields 2 (Array.length fields - 2))
    (fun fields -> Node.outputs types (IR_LuaJIT.caught fields.(1) payload_types))

let lower options types node =
  match node.kind with
  (* LuaJIT: I64s lower to FFI longs, NaN tagging reads other NaNs as other values, and FFI accesses are unchecked. *)
  | Operator (IntegerBinary (width, operator)) -> Some (integer_binary types node width operator)
  | Operator (IntegerCount (width, operator)) -> integer_count node width operator
  | Operator (IntegerComparison (width, operator)) -> Some (integer_comparison types node width operator)
  | Operator (NumberBinary (width, operator)) -> Floats.binary types width operator node.inputs.(0) node.inputs.(1)
  | Operator (NumberUnary (width, operator)) -> Floats.unary types width operator node.inputs.(0)
  | Operator (NumberComparison (width, operator)) ->
      (* LuaJIT: I64s lower to FFI longs, NaN tagging reads other NaNs as other values, and FFI accesses are unchecked. *)
      Some (Floats.compare types width operator node.inputs.(0) node.inputs.(1))
  | ArraySame | BoxSame -> Some (extended `LuaJITEqual node.inputs)
  | Operator (Conversion operator) -> conversion types node operator
  | ArrayGet _ -> Some (Storage.get options types node)
  | ArraySet _ -> Some (Storage.set options types node)
  | ArrayFill _ -> Some (Storage.fill options types node)
  | ArrayCopy _ -> Some (Storage.copy options types node)
  | ArrayNew _ -> Some (Storage.allocate types node)
  | FunctionTry payload_types -> Some (attempt types node payload_types)
  | ArrayOf { element_type; bytes; array_type } ->
      Some
        (if element_type = Type.i8 then extended (`LuaJITBytesFromString bytes) [||]
         else extended (`LuaJITArrayOf (array_type, element_type, bytes)) [||])
  | ArraySize -> Some (Storage.length types node.inputs.(0))
  | BoxNew box_type -> Some (extended (`LuaJITBoxNew box_type) node.inputs)
  | BoxGet _ -> Some (extended (`LuaJITBoxGet (Node.operand_type types node)) node.inputs)
  | BoxSet box_type -> Some (extended (`LuaJITBoxSet box_type) node.inputs)
  | BoxRetype target | FunctionRetype target -> Some (extended (`LuaJITRetype target) node.inputs)
  | ArrayFinalize _ | BoxFinalize _ -> Some node.inputs.(0)
  (* LuaJIT: I64s lower to FFI longs, NaN tagging reads other NaNs as other values, and FFI accesses are unchecked. *)
  | Extension (#IR_Memory.store as access) -> Some (Storage.store options types node access)
  | Extension (#IR_Memory.load as access) -> Some (Storage.load options types node access)
  | _ -> None

let run options (compilation_unit : x CompilationUnit.t) =
  ExtensionLowering.run (lower options compilation_unit.types) compilation_unit;
  Optimizer.settle options
    (* LuaJIT: I64s lower to FFI longs, NaN tagging reads other NaNs as other values, and FFI accesses are unchecked. *)
    (fun types _ -> [ LuaJITFolding.fold options types; LuaJITIdentities.fold options types ])
    compilation_unit

let optimizations =
  [
    LuaJITFolding.bit_operations;
    (* LuaJIT: I64s lower to FFI longs, NaN tagging reads other NaNs as other values, and FFI accesses are unchecked. *)
    LuaJITFolding.arithmetic;
    LuaJITFolding.longs;
    LuaJITFolding.quotients;
    LuaJITFolding.views;
    LuaJITIdentities.bit_operations;
    LuaJITIdentities.views;
    LuaJITIdentities.ranges;
    LuaJITIdentities.longs;
    (* LuaJIT: I64s lower to FFI longs, NaN tagging reads other NaNs as other values, and FFI accesses are unchecked. *)
    LuaJITIdentities.f64_bits;
    LuaJITIdentities.writes;
    Storage.omission;
  ]
