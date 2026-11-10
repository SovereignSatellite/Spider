open IR

exception Unsupported of string

(** The module does not decode or validate, its custom sections included. *)
exception Invalid of string

val unsupported : Wasm.Ast.instr' -> 'result

type signature = { parameters : Type.t list; results : Type.t list }

val index : Wasm.Ast.idx -> int
val composites : Wasm.Ast.type_ list -> Wasm.Types.comptype array

type signatures = {
  composites : Wasm.Types.comptype array;
  types : signature Lazy.t array;
  functions : signature array;
  globals : Type.t array;
  mutable_ : bool array;
  tags : Type.t list array;
  tables : Type.t array;
  elements : Type.t array;
  reference : Type.t;
  exception_ : Type.t;
  host : Type.t;
}

val value_type : signatures -> Wasm.Types.valtype -> Type.t
val imported : (Wasm.Types.externtype -> 'item option) -> Wasm.Ast.module_ -> 'item list
val imported_function : Wasm.Types.externtype -> int option
val imported_memory : Wasm.Types.externtype -> Wasm.Types.memorytype option
val table_types : Wasm.Ast.module_ -> Wasm.Types.tabletype list
val signatures : Type.t -> Type.t -> Type.t -> Wasm.Ast.module_ -> signatures
