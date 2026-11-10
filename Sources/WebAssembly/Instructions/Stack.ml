open IR
open Types

let takes parameters results = { parameters; results }

let of_block signatures (block_type : Wasm.Ast.blocktype) =
  match block_type with
  | VarBlockType type_index -> Lazy.force signatures.types.(index type_index)
  | ValBlockType None -> takes [] []
  | ValBlockType (Some result) -> (
      match value_type signatures result with
      | result_type -> takes [] [ result_type ]
      | exception Unsupported _ -> raise (Unsupported (Wasm.Types.string_of_valtype result ^ " blocks")))

let unary operand result = takes [ operand ] [ result ]
let binary operand result = takes [ operand; operand ] [ result ]

let number (value_type : Wasm.Types.numtype) =
  match value_type with I32T -> Type.i32 | I64T -> Type.i64 | F32T -> Type.f32 | F64T -> Type.f64

let typed (operation : ('i32, 'i64, 'f32, 'f64) Wasm.Value.op) =
  match operation with I32 _ -> Type.i32 | I64 _ -> Type.i64 | F32 _ -> Type.f32 | F64 _ -> Type.f64

let integer_conversion result (operator : Wasm.Ast.IntOp.cvtop) =
  let from operand = unary (Type.of_scalar operand) (Type.of_scalar result) in
  match operator with
  | ExtendI32 _ -> from `I32
  | WrapI64 -> from `I64
  | TruncF32 _ | TruncSatF32 _ -> from `F32
  | TruncF64 _ | TruncSatF64 _ -> from `F64
  | ReinterpretFloat -> from (match result with `I32 -> `F32 | `I64 -> `F64)

let number_conversion result (operator : Wasm.Ast.FloatOp.cvtop) =
  let from operand = unary (Type.of_scalar operand) (Type.of_scalar result) in
  match operator with
  | ConvertI32 _ -> from `I32
  | ConvertI64 _ -> from `I64
  | PromoteF32 -> from `F32
  | DemoteF64 -> from `F64
  | ReinterpretInt -> from (match result with `F32 -> `I32 | `F64 -> `I64)

let conversion (operation : Wasm.Ast.cvtop) =
  match operation with
  | I32 operator -> integer_conversion `I32 operator
  | I64 operator -> integer_conversion `I64 operator
  | F32 operator -> number_conversion `F32 operator
  | F64 operator -> number_conversion `F64 operator

let called_through signatures type_index through =
  let called = Lazy.force signatures.types.(index type_index) in
  takes (called.parameters @ [ through ]) called.results

let change signatures (instruction : Wasm.Ast.instr') =
  let reference = signatures.reference in
  match instruction with
  | Call called -> signatures.functions.(index called)
  | CallIndirect (_, type_index) -> called_through signatures type_index Type.i32
  | CallRef type_index -> called_through signatures type_index reference
  | RefNull heap -> takes [] [ value_type signatures (RefT (Null, heap)) ]
  | RefFunc _ -> takes [] [ reference ]
  | GlobalGet global -> takes [] [ signatures.globals.(index global) ]
  | GlobalSet global -> takes [ signatures.globals.(index global) ] []
  | Unreachable | DataDrop _ | ElemDrop _ -> takes [] []
  | Throw tag -> takes signatures.tags.(index tag) []
  | ThrowRef -> takes [ signatures.exception_ ] []
  | TableGet table -> takes [ Type.i32 ] [ signatures.tables.(index table) ]
  | TableSet table -> takes [ Type.i32; signatures.tables.(index table) ] []
  | TableSize _ | MemorySize _ -> takes [] [ Type.i32 ]
  | TableGrow table -> takes [ signatures.tables.(index table); Type.i32 ] [ Type.i32 ]
  | TableFill table -> takes [ Type.i32; signatures.tables.(index table); Type.i32 ] []
  | TableCopy _ | TableInit _ | MemoryInit _ | MemoryCopy _ | MemoryFill _ -> takes [ Type.i32; Type.i32; Type.i32 ] []
  | Const constant -> takes [] [ typed constant.it ]
  | MemoryGrow _ -> unary Type.i32 Type.i32
  | Load (_, { ty; _ }) -> unary Type.i32 (number ty)
  | Store (_, { ty; _ }) -> takes [ Type.i32; number ty ] []
  | Test operation -> unary (typed operation) Type.i32
  | Compare operation -> binary (typed operation) Type.i32
  | Unary operation -> unary (typed operation) (typed operation)
  | Binary operation -> binary (typed operation) (typed operation)
  | Convert operation -> conversion operation
  | other -> unsupported other
