open IR

type t = {
  locals : Type.t array;
  result_types : Type.t list;
  thrown : Type.t;
  kinds : Type.t array;
  first_slot : int;
  mutable count : int;
  results : int list;
}

val slot : t -> int -> Type.t -> int

(** Each kind of reference has slots of its own, and the thrown exception is one of them. *)
val create : Type.t array -> Type.t list -> Type.t list -> Type.t -> t

val local : int -> int
val type_ : t -> int -> Type.t
