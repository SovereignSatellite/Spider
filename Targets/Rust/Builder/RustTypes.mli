open IR
open Allocator
open Emission
open Tree
open RustExtension

type t

(** The field of a box holding the leaf of this index. *)
val leaf_field : int -> Name.t

val create : x TypeArena.t -> x Layouts.t -> RuntimeLibrary.NeededHelpers.t -> t
val leaf_type : t -> Type.t -> RustType.t
val result_type : t -> Type.t list -> RustType.t
val zero : t -> Type.t -> Expression.t
val element_pointer : t -> Type.t -> RustType.t
val box_name : t -> Type.t -> Name.t
val definitions : t -> Item.t list
