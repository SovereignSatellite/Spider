open IR
open Definitions
open Entities

val position : Fields.key array -> Fields.key -> int

(** For each field in [ordered], [contents] holds its contents after their last write, which reads take, and [pending] a
    [Unit] link after every access since the start, which writes and the end wait for. [after] and [replaced] update the
    given state's arrays in place, so a state given to them is not used again. *)
type 'x t = {
  token : 'x link;
  uses : Field.t array;
  held : 'x link array;
  ordered : Field.t array;
  contents : 'x link array;
  pending : 'x link array;
}

val item : 'x t -> Fields.key -> 'x link
val links : 'x t -> 'x link array
val of_links : 'x t -> 'x link array -> 'x t
val of_map : 'x context -> 'x link -> 'x link Fields.t -> Fields.key array -> 'x t
val enter : 'x context -> int -> Fields.key list -> Type.t -> 'x link -> 'x link -> 'x t
val contents : 'x context -> 'x t -> Fields.key -> Access.t -> 'x link
val box : 'x t -> Fields.key -> 'x link
val after : 'x t -> Fields.key -> Access.t -> 'x link -> 'x t
val replaced : 'x t -> Fields.key -> 'x link -> 'x link -> 'x t

(** Each field's contents are read from its box again, since what the call that threw wrote never returned. *)
val unwound : 'x context -> 'x t -> 'x link -> 'x t

val settle : 'x t -> 'x t
