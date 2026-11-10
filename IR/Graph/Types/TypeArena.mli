open Definitions

type 'x t = 'x type_arena

val create : 'x extension -> 'x t

(** May run on several domains at once, as may [extension], [definition] and [describe], except while [create_box] runs.
*)
val intern : 'x t -> StructuralType.t -> Type.t

val extension : 'x t -> TypeDefinition.extension -> Type.t

(** A new box, whose contents may hold the box itself. *)
val create_box : 'x t -> (Type.t -> Type.t) -> Type.t

val definition : 'x t -> Type.t -> TypeDefinition.t
val describe : 'x t -> Type.t -> string
val describe_all : 'x t -> Type.t list -> string
val element : 'x t -> Type.t -> Type.t
val contents : 'x t -> Type.t -> Type.t
val signature : 'x t -> Type.t -> Type.t list * Type.t list
val variants : 'x t -> Type.t -> Type.t list list
