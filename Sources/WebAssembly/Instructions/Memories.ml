open IR
open Definitions
open Entities
open Accesses

let memory index : Field.t = Memory (Types.index index)
let pages memory = make (Operator (IntegerBinary (`I32, ShiftRightU))) [| make ArraySize [| memory |]; i32 16 |]
let size context state index = read context state (memory index) pages

let grow context state index delta =
  Growth.grow context state (memory index) pages delta (make (I8Constant 0) [||]) (fun pages ->
      make (Operator (IntegerBinary (`I32, ShiftLeft))) [| pages; i32 16 |])

let drop context state segment = empty context state (Data segment) (result (ArrayOf.create context.types Type.i8 ""))

let extended width from (signedness : Wasm.Pack.sx) value =
  match signedness with U -> value | S -> converted (SignExtend (width, from)) value

let loaded memory address (load : Wasm.Ast.loadop) =
  let offset = Int64.to_int load.offset in
  let loaded kind = make (Extension kind) [| memory; address |] in
  let byte () = converted (Widen `I8) (loaded (`MemoryLoadI8 offset)) in
  let half () = converted (Widen `I16) (loaded (`MemoryLoadI16 offset)) in
  let word () = loaded (`MemoryLoadI32 offset) in
  let double () = loaded (`MemoryLoadI64 offset) in
  match (load.ty, load.pack) with
  | I32T, None -> word ()
  | I64T, None -> double ()
  | F32T, None -> converted (Retype `F32) (word ())
  | F64T, None -> converted (Retype `F64) (double ())
  | I32T, Some (Pack8, signedness) -> extended `I32 `I8 signedness (byte ())
  | I32T, Some (Pack16, signedness) -> extended `I32 `I16 signedness (half ())
  | I64T, Some (Pack8, signedness) -> extended `I64 `I8 signedness (converted (Widen `I32) (byte ()))
  | I64T, Some (Pack16, signedness) -> extended `I64 `I16 signedness (converted (Widen `I32) (half ()))
  | I64T, Some (Pack32, signedness) -> extended `I64 `I32 signedness (converted (Widen `I32) (word ()))
  | I32T, Some ((Pack32 | Pack64), _) | I64T, Some (Pack64, _) | (F32T | F64T), Some _ ->
      invalid_arg "No instruction loads this way."

let stored memory address (store : Wasm.Ast.storeop) value =
  let offset = Int64.to_int store.offset in
  let stored kind value = make (Extension kind) [| memory; address; value |] in
  match (store.ty, store.pack) with
  | I32T, None -> stored (`MemoryStoreI32 offset) value
  | I64T, None -> stored (`MemoryStoreI64 offset) value
  | F32T, None -> stored (`MemoryStoreI32 offset) (converted (Retype `I32) value)
  | F64T, None -> stored (`MemoryStoreI64 offset) (converted (Retype `I64) value)
  | I32T, Some Pack8 -> stored (`MemoryStoreI8 offset) (converted (Narrow `I8) value)
  | I32T, Some Pack16 -> stored (`MemoryStoreI16 offset) (converted (Narrow `I16) value)
  | I64T, Some Pack8 -> stored (`MemoryStoreI8 offset) (converted (Narrow `I8) (converted (Narrow `I32) value))
  | I64T, Some Pack16 -> stored (`MemoryStoreI16 offset) (converted (Narrow `I16) (converted (Narrow `I32) value))
  | I64T, Some Pack32 -> stored (`MemoryStoreI32 offset) (converted (Narrow `I32) value)
  | I32T, Some (Pack32 | Pack64) | I64T, Some Pack64 | (F32T | F64T), Some _ ->
      invalid_arg "No instruction stores this way."

let load context state index operation address =
  read context state (memory index) (fun bytes -> loaded bytes address operation)

let store context state index operation address value =
  write context state (memory index) (fun bytes -> stored bytes address operation value)
