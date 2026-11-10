(** An access past the end of a memory, a table or a segment is undefined, whereas WebAssembly traps. *)

open IR
open Definitions
open Types

module Field : sig
  type t = Function of int | Memory of int | Data of int | Table of int | Element of int | Global of int | Tag of int

  val compare : t -> t -> int
end

module Fields : Map.S with type key = Field.t

(** Each case covers those before it: reading a field's contents, writing them, and replacing what its box holds. *)
module Access : sig
  type t = Reads | Writes | Replaces
end

module Callee : sig
  type t = Function of int | Indirect of signature | Host
end

type 'x context = {
  types : 'x TypeArena.t;
  signatures : signatures;
  erased : Type.t;
  imported_functions : int;
  bodies : 'x region array;
  tag_box : Type.t;
  exception_box : Type.t;
  boxes : Type.t -> Type.t;
  effects : (Callee.t, Access.t Fields.t) Hashtbl.t;
}

val result : 'x node -> 'x link
val make : 'x kind -> 'x link array -> 'x link
val converted : Conversion.t -> 'x link -> 'x link
val fence : 'x link -> 'x link array -> 'x link
val i32 : int -> 'x link
val distinct : 'item list -> 'item list

module Context : sig
  val contents : 'x context -> Type.t -> Type.t
  val function_type : 'x context -> signature -> Type.t
  val function_of : 'x context -> int -> Type.t
  val erased_function : 'x type_arena -> Type.t
  val closure : 'x context -> Type.t -> Type.t
  val field_type : 'x context -> Field.t -> Type.t
  val thrown : 'x context -> int -> int
end

val limited : 'x context -> Field.t -> 'x link -> 'x link -> 'x link
val limited_parts : 'x context -> 'x link -> 'x node
val held : 'x context -> Field.t -> 'x link -> 'x link
val zero : Type.t -> 'x link
val select : 'x type_arena -> int -> 'x link -> 'x link array -> 'x link
