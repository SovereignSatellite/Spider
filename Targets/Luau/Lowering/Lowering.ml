open IR
open Definitions
open Passes
open LuauExtension
open Numbers

let integer_binary types node width operator =
  let left, right = (node.inputs.(0), node.inputs.(1)) in
  match width with
  (* Luau: the I64 lowering may leave an operation a prefab call, while the I32 lowering never does. *)
  | `I32 -> Some (Integers.binary operator left right)
  | `I64 -> Words.binary types operator left right

(* Luau: counting an I64 as two words builds Matches, which need the types. *)
let integer_count types node width operator =
  let value = node.inputs.(0) in
  match width with `I32 -> Integers.count operator value | `I64 -> Words.count types operator value

let integer_comparison types node width operator =
  let left, right = (node.inputs.(0), node.inputs.(1)) in
  match width with `I32 -> Integers.compare operator left right | `I64 -> Words.compare types operator left right

let sign_extend width from value =
  match width with
  | `I32 -> Integers.sign_extend (Scalar.bits from) value
  | `I64 -> Words.sign_extend (Scalar.bits from) value

let narrow_i32 target value =
  retype
    (target :> Scalar.t)
    (extended `LuauBit32Band [| to_number value; constant (Int64.to_float (Constant.mask (Type.of_scalar target))) |])

let widen source value =
  match source with `F32 -> Floats.widen value | `I32 -> Words.widen value | `I8 | `I16 -> retype `I32 value

let narrow target value =
  match target with
  | `F32 -> Floats.narrow value
  | `I32 -> Words.narrow value
  | (`I8 | `I16) as target -> narrow_i32 target value

let conversion types node (conversion : Conversion.t) =
  let value = node.inputs.(0) in
  match conversion with
  | Widen source -> Some (widen source value)
  | Narrow target -> Some (narrow target value)
  | SignExtend (width, from) -> Some (sign_extend width from value)
  | ConvertSigned target -> Floats.convert_signed target value
  | ConvertUnsigned target -> Floats.convert_unsigned target value
  | SaturateSigned target -> Some (Floats.saturate_signed types target value)
  | SaturateUnsigned target -> Some (Floats.saturate_unsigned types target value)
  (* Luau: I64s lower to two I32 words, F32s compute on F64s, buffers check their own bounds, and reads fold through their readers. *)
  | Retype target -> Some (retype (target :> Scalar.t) value)

let attempt types attempt payload_types =
  ExtensionLowering.attempt types attempt
    (fun inputs -> Node.outputs types (IR_Luau.try_ inputs.(0) (Array.sub inputs 1 (Array.length inputs - 1))))
    Fun.id
    (fun fields -> Array.sub fields 2 (Array.length fields - 2))
    (fun fields -> Node.outputs types (IR_Luau.caught fields.(1) payload_types))

let lower options types node =
  match node.kind with
  | Operator (IntegerBinary (width, operator)) -> integer_binary types node width operator
  (* Luau: I64s lower to two I32 words, F32s compute on F64s, buffers check their own bounds, and reads fold through their readers. *)
  | Operator (IntegerCount (width, operator)) -> Some (integer_count types node width operator)
  | Operator (IntegerComparison (width, operator)) -> Some (integer_comparison types node width operator)
  | Operator (NumberBinary (width, operator)) -> Floats.binary types width operator node.inputs.(0) node.inputs.(1)
  | Operator (NumberUnary (width, operator)) -> Floats.unary types width operator node.inputs.(0)
  | Operator (NumberComparison (width, operator)) ->
      Some (Floats.compare width operator node.inputs.(0) node.inputs.(1))
  | ArraySame | BoxSame -> Some (extended `LuauEqual node.inputs)
  | Operator (Conversion operator) -> conversion types node operator
  | ArrayGet _ -> Some (Storage.get options types node)
  | ArraySet _ -> Some (Storage.set options types node)
  | ArrayFill _ -> Some (Storage.fill options types node)
  | ArrayCopy _ -> Some (Storage.copy options types node)
  | ArrayNew _ -> Some (Storage.allocate types node)
  | FunctionTry payload_types -> Some (attempt types node payload_types)
  | ArrayOf { element_type; bytes; array_type } ->
      Some
        (if element_type = Type.i8 then extended (`LuauBufferFromString bytes) [||]
         else extended (`LuauArrayOf (array_type, element_type, bytes)) [||])
  | ArraySize -> Some (Storage.length types node.inputs.(0))
  | BoxNew box_type -> Some (extended (`LuauBoxNew box_type) node.inputs)
  | BoxGet _ -> Some (extended (`LuauBoxGet (Node.operand_type types node)) node.inputs)
  | BoxSet box_type -> Some (extended (`LuauBoxSet box_type) node.inputs)
  | BoxRetype target | FunctionRetype target -> Some (extended (`LuauRetype target) node.inputs)
  | ArrayFinalize _ | BoxFinalize _ -> Some node.inputs.(0)
  | Extension (#IR_Memory.store as access) -> Some (Storage.store node access)
  | Extension (#IR_Memory.load as access) -> Some (Storage.load node access)
  | _ -> None

let run options (compilation_unit : x CompilationUnit.t) =
  ExtensionLowering.run (lower options compilation_unit.types) compilation_unit;
  Optimizer.settle options
    (* Luau: I64s lower to two I32 words, F32s compute on F64s, buffers check their own bounds, and reads fold through their readers. *)
    (fun types table -> [ LuauFolding.fold options types; LuauIdentities.fold options types table ])
    compilation_unit

let optimizations =
  [
    LuauFolding.bit_operations;
    LuauFolding.arithmetic;
    LuauFolding.quotients;
    LuauFolding.views;
    LuauIdentities.bit_operations;
    LuauIdentities.views;
    LuauIdentities.ranges;
    LuauIdentities.words;
    (* Luau: I64s lower to two I32 words, F32s compute on F64s, buffers check their own bounds, and reads fold through their readers. *)
    LuauIdentities.writes;
    LuauIdentities.reads;
    Storage.omission;
  ]
