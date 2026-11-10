(* Luau: Luau is typed. *)
open IR
open Allocator
open Emission
open Tree
open LuauExtension

type t

val create : x TypeArena.t -> x Layouts.t -> RuntimeLibrary.NeededHelpers.t -> t
val leaf_type : t -> Type.t -> LuauType.t
val tag : t -> Type.t -> string
val definitions : t -> Statement.t list
