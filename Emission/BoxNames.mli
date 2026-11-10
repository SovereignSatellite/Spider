open IR

type 'definition t

val create : unit -> 'definition t
val definitions : 'definition t -> (Name.t * 'definition) list

(** [definition_of] may name boxes, this one among them. *)
val name : 'definition t -> Type.t -> (Type.t -> 'definition) -> Name.t
