open IR
open Definitions
open Types
open Entities
open Accesses
open Calls

let computed state result = ([ result ], state)
let trapping (state : 'x State.t) result = ([ result ], { state with token = fence state.token [| result |] })

let compute_numeric context state instruction operands =
  match Numbers.traps_on context instruction with
  | Some build -> trapping state (build operands)
  | None -> computed state (Numbers.numeric instruction operands)

let constant (value : Wasm.Value.num) =
  match value with
  | I32 value -> i32 (Int32.to_int value)
  | I64 value -> make (I64Constant value) [||]
  | F32 value -> make (F32Constant (Wasm.F32.to_bits value)) [||]
  | F64 value -> make (F64Constant (Int64.float_of_bits (Wasm.F64.to_bits value))) [||]

let lift context state (instruction : Wasm.Ast.instr') operands =
  let operand position = List.nth operands position in
  match instruction with
  | Const value -> computed state (constant value.it)
  | Unreachable -> Exceptions.raise state (zero context.signatures.exception_)
  | Throw tag -> Exceptions.raise state (Exceptions.exception_ context state (index tag) operands)
  | ThrowRef -> Exceptions.raise state (operand 0)
  | Call called -> call context state (index called) operands
  | CallIndirect (table, type_index) -> Tables.call_indirect context state type_index table operands
  | CallRef type_index -> Tables.call_reference context state type_index operands
  | GlobalGet global -> read context state (Global (index global)) (fun box -> result (BoxGet.create context.types box))
  | GlobalSet global -> ([], set context state (Global (index global)) (operand 0))
  | RefNull _ -> computed state (zero (List.hd (Stack.change context.signatures instruction).results))
  | RefFunc referenced -> computed state (reference context state (index referenced))
  | TableGet table -> Tables.get context state table operands
  | TableSet table -> Tables.set context state table operands
  | Load (memory, operation) -> Memories.load context state memory operation (operand 0)
  | Store (memory, operation) -> Memories.store context state memory operation (operand 0) (operand 1)
  | MemoryInit (memory, segment) -> copy context state (Memories.memory memory) (Data (index segment)) operands
  | DataDrop segment -> Memories.drop context state (index segment)
  | MemoryFill memory ->
      fill context state (Memories.memory memory) [ operand 0; converted (Narrow `I8) (operand 1); operand 2 ]
  | MemoryCopy (destination, source) ->
      copy context state (Memories.memory destination) (Memories.memory source) operands
  | TableSize table -> Tables.size context state table
  | TableGrow table -> Tables.grow context state table operands
  | TableFill table -> fill context state (Tables.table table) operands
  | TableCopy (destination, source) -> copy context state (Tables.table destination) (Tables.table source) operands
  | TableInit (table, segment) -> copy context state (Tables.table table) (Element (index segment)) operands
  | ElemDrop segment -> Tables.drop context state (index segment)
  | MemorySize memory -> Memories.size context state memory
  | MemoryGrow memory -> Memories.grow context state memory (operand 0)
  | RefIsNull -> computed state (Tables.is_null (operand 0))
  | RefAsNonNull -> trapping state (Tables.non_null context (operand 0))
  | _ -> compute_numeric context state instruction operands

let copies destination source : (Field.t * Access.t) list = [ (destination, Writes); (source, Reads) ]

let caught_tag (caught : Wasm.Ast.catch) : (Field.t * Access.t) list =
  match caught.it with
  | Catch (tag, _) | CatchRef (tag, _) -> [ (Tag (index tag), Reads) ]
  | CatchAll _ | CatchAllRef _ -> []

let accesses (instruction : Wasm.Ast.instr') : (Field.t * Access.t) list =
  match instruction with
  | Call called -> [ (Function (index called), Reads) ]
  | CallIndirect (table, _) -> [ (Tables.table table, Reads) ]
  | GlobalGet global -> [ (Global (index global), Reads) ]
  | GlobalSet global -> [ (Global (index global), Writes) ]
  | RefFunc referenced -> [ (Function (index referenced), Reads) ]
  | TableGet table -> [ (Tables.table table, Reads) ]
  | TableSet table -> [ (Tables.table table, Writes) ]
  | Load (memory, _) -> [ (Memories.memory memory, Reads) ]
  | Store (memory, _) -> [ (Memories.memory memory, Writes) ]
  | MemorySize memory -> [ (Memories.memory memory, Reads) ]
  | MemoryGrow memory -> [ (Memories.memory memory, Replaces) ]
  | MemoryInit (memory, segment) -> copies (Memories.memory memory) (Data (index segment))
  | DataDrop segment -> [ (Data (index segment), Replaces) ]
  | MemoryFill memory -> [ (Memories.memory memory, Writes) ]
  | MemoryCopy (destination, source) -> copies (Memories.memory destination) (Memories.memory source)
  | TableSize table -> [ (Tables.table table, Reads) ]
  | TableGrow table -> [ (Tables.table table, Replaces) ]
  | TableFill table -> [ (Tables.table table, Writes) ]
  | TableCopy (destination, source) -> copies (Tables.table destination) (Tables.table source)
  | TableInit (table, segment) -> copies (Tables.table table) (Element (index segment))
  | ElemDrop segment -> [ (Element (index segment), Replaces) ]
  | Throw tag -> [ (Tag (index tag), Reads) ]
  | TryTable (_, catches, _) -> List.concat_map caught_tag catches
  | Unreachable | Nop | Drop | Select _ | Block _ | Loop _ | If _ | Br _ | BrIf _ | BrTable _ | BrOnNull _
  | BrOnNonNull _ | BrOnCast _ | BrOnCastFail _ | Return | CallRef _ | ReturnCall _ | ReturnCallRef _
  | ReturnCallIndirect _ | ThrowRef | LocalGet _ | LocalSet _ | LocalTee _ | VecLoad _ | VecStore _ | VecLoadLane _
  | VecStoreLane _ | RefNull _ | RefIsNull | RefAsNonNull | RefTest _ | RefCast _ | RefEq | RefI31 | I31Get _
  | StructNew _ | StructGet _ | StructSet _ | ArrayNew _ | ArrayNewFixed _ | ArrayNewData _ | ArrayNewElem _
  | ArrayGet _ | ArraySet _ | ArrayLen | ArrayCopy _ | ArrayFill _ | ArrayInitData _ | ArrayInitElem _ | ExternConvert _
  | Const _ | Test _ | Compare _ | Unary _ | Binary _ | Convert _ | VecConst _ | VecTest _ | VecCompare _ | VecUnary _
  | VecBinary _ | VecTernary _ | VecConvert _ | VecShift _ | VecBitmask _ | VecTestBits _ | VecUnaryBits _
  | VecBinaryBits _ | VecTernaryBits _ | VecSplat _ | VecExtract _ | VecReplace _ ->
      []

let throws (instruction : Wasm.Ast.instr') =
  match instruction with
  | Call _ | CallIndirect _ | CallRef _ | Throw _ | ThrowRef -> true
  | Unreachable | Nop | Drop | Select _ | Block _ | Loop _ | If _ | Br _ | BrIf _ | BrTable _ | BrOnNull _
  | BrOnNonNull _ | BrOnCast _ | BrOnCastFail _ | Return | ReturnCall _ | ReturnCallRef _ | ReturnCallIndirect _
  | TryTable _ | LocalGet _ | LocalSet _ | LocalTee _ | GlobalGet _ | GlobalSet _ | TableGet _ | TableSet _
  | TableSize _ | TableGrow _ | TableFill _ | TableCopy _ | TableInit _ | ElemDrop _ | Load _ | Store _ | VecLoad _
  | VecStore _ | VecLoadLane _ | VecStoreLane _ | MemorySize _ | MemoryGrow _ | MemoryFill _ | MemoryCopy _
  | MemoryInit _ | DataDrop _ | RefNull _ | RefFunc _ | RefIsNull | RefAsNonNull | RefTest _ | RefCast _ | RefEq
  | RefI31 | I31Get _ | StructNew _ | StructGet _ | StructSet _ | ArrayNew _ | ArrayNewFixed _ | ArrayNewData _
  | ArrayNewElem _ | ArrayGet _ | ArraySet _ | ArrayLen | ArrayCopy _ | ArrayFill _ | ArrayInitData _ | ArrayInitElem _
  | ExternConvert _ | Const _ | Test _ | Compare _ | Unary _ | Binary _ | Convert _ | VecConst _ | VecTest _
  | VecCompare _ | VecUnary _ | VecBinary _ | VecTernary _ | VecConvert _ | VecShift _ | VecBitmask _ | VecTestBits _
  | VecUnaryBits _ | VecBinaryBits _ | VecTernaryBits _ | VecSplat _ | VecExtract _ | VecReplace _ ->
      false
