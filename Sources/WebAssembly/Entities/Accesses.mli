open IR
open Definitions
open Entities

val read : 'x context -> 'x State.t -> Fields.key -> ('x link -> 'x link) -> 'x link list * 'x State.t
val write : 'x context -> 'x State.t -> Fields.key -> ('x link -> 'x link) -> 'x link list * 'x State.t
val copy : 'x context -> 'x State.t -> Fields.key -> Fields.key -> 'x link list -> 'x link list * 'x State.t
val fill : 'x context -> 'x State.t -> Fields.key -> 'x link list -> 'x link list * 'x State.t
val empty : 'x context -> 'x State.t -> Fields.key -> 'x link -> 'x link list * 'x State.t
val set : 'x context -> 'x State.t -> Fields.key -> 'x link -> 'x State.t
