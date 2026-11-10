open IR
open Definitions
open Entities

(** A throw's payload is the exception and the token it was thrown at, so a throw follows what came before it. *)
val payload_types : 'x context -> Type.t list

val traps_in : 'x context -> 'x link -> int -> Type.t list -> ('x link array -> 'x link array) -> 'x node
val allocate : 'x context -> 'x link -> 'x link -> 'x link
