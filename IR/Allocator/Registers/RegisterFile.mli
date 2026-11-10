open IR

(** [registers] fixed registers, each taking the class of the first value it holds; a value given none goes to the
    unbounded pool of its class, whose slots no other pool's share. *)
type t = { registers : int; class_ : Type.t -> int }

val create : int -> (Type.t -> 'key) -> t
