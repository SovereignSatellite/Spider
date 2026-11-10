open IR

type layout

(** The arrays it returns are its own, which no caller may write. *)
type 'x t

val types : 'x t -> 'x TypeArena.t
val create : 'x TypeArena.t -> 'x t
val leaves_of : 'x t -> Type.t -> Type.t array
val leaf_count : 'x t -> Type.t -> int
val tagged : 'x t -> Type.t -> bool
val leaves : 'x t -> Type.t list -> Type.t list
val select : 'x t -> Type.t -> Path.t -> int array
