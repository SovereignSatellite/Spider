(** The host may replace the memory and tables the module shares, write the mutable globals it shares, and call its
    exports and, once its interface can hand the host a function reference, every function the module references. *)

open Types
open Entities

val exported : Wasm.Ast.export -> Fields.key

type imported = { functions : int; memories : int; tables : int; globals : int; tags : int }

val imports : Wasm.Ast.module_ -> imported * Fields.key list
val accessed : Wasm.Ast.func -> Access.t Fields.t
val effects : signatures -> Wasm.Ast.module_ -> (Callee.t, Access.t Fields.t) Hashtbl.t
