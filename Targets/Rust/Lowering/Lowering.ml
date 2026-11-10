open IR
open Definitions
open Passes
open RustExtension
open Numbers

let conversion types node (conversion : Conversion.t) =
  let value = node.inputs.(0) in
  match conversion with
  | SignExtend (width, from) -> Integers.sign_extend types width from value
  | ConvertSigned target -> Floats.convert_signed types target value
  | SaturateSigned target -> Floats.saturate_signed types target value
  | Widen _ | Narrow _ | ConvertUnsigned _ | SaturateUnsigned _ ->
      cast (Type.of_scalar (Conversion.target conversion)) value
  | Retype target -> Floats.retype target value

let attempt types attempt payload_types =
  ExtensionLowering.attempt types attempt
    (fun inputs -> Node.outputs types (IR_Rust.try_ inputs.(0) (Array.sub inputs 1 (Array.length inputs - 1))))
    (fun outcome -> Link.output (IR_Rust.threw outcome) 0)
    (fun fields -> Node.outputs types (IR_Rust.returned fields.(0)))
    (fun fields -> Node.outputs types (IR_Rust.caught fields.(0) payload_types))

let lower options types node =
  match node.kind with
  | Operator (IntegerBinary (width, operator)) ->
      Some (Integers.binary types width operator node.inputs.(0) node.inputs.(1))
  | Operator (IntegerCount (width, operator)) -> Some (Integers.count width operator node.inputs.(0))
  | Operator (IntegerComparison (width, operator)) ->
      Some (Integers.compare types width operator node.inputs.(0) node.inputs.(1))
  | Operator (Conversion operator) -> Some (conversion types node operator)
  | Operator (NumberBinary (_, operator)) -> Floats.binary operator node.inputs.(0) node.inputs.(1)
  | Operator (NumberUnary (_, operator)) -> Some (Floats.unary operator node.inputs.(0))
  | Operator (NumberComparison (_, operator)) -> Some (Floats.compare operator node.inputs.(0) node.inputs.(1))
  | ArraySame | BoxSame -> Some (extended `RustPointerEqual node.inputs)
  | ArrayGet _ -> Some (Storage.get options types node)
  | ArraySet _ -> Some (Storage.set options types node)
  | ArraySize -> Some (Storage.size types node)
  | ArrayFill _ -> Some (Storage.fill options types node)
  | ArrayCopy _ -> Some (Storage.copy options types node)
  | ArrayNew _ -> Some (Storage.allocate types node)
  | FunctionTry payload_types -> Some (attempt types node payload_types)
  | ArrayOf { element_type; bytes; array_type } -> Some (extended (`RustArrayOf (array_type, element_type, bytes)) [||])
  | BoxNew box_type -> Some (extended (`RustBoxNew box_type) node.inputs)
  | BoxGet _ -> Some (extended (`RustBoxGet (Node.operand_type types node)) node.inputs)
  | BoxSet box_type -> Some (extended (`RustBoxSet box_type) node.inputs)
  | BoxRetype target | FunctionRetype target -> Some (extended (`RustRetype target) node.inputs)
  | Extension (#IR_Memory.store as access) -> Some (Storage.store options types node access)
  | Extension (#IR_Memory.load as access) -> Some (Storage.load options types node access)
  | _ -> None

let run options (compilation_unit : x CompilationUnit.t) =
  ExtensionLowering.run (lower options compilation_unit.types) compilation_unit;
  Optimizer.settle options
    (fun types _ -> [ RustFolding.fold options types; RustIdentities.fold options types ])
    compilation_unit

let optimizations = [ RustFolding.casts; RustFolding.sums; RustIdentities.casts; RustIdentities.sums; Storage.omission ]
