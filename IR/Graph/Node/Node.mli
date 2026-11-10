open Definitions

type 'x t = 'x node

val next : int Atomic.t
val create : 'x kind -> 'x link array -> 'x t
val inputs : 'x t -> 'x link array
val exists : ('x link -> bool) -> 'x t -> bool
val for_all : ('x link -> bool) -> 'x t -> bool
val reading : 'x link array -> 'x t -> 'x t
val name : 'x type_arena -> 'x t -> string
val same_kind : 'x type_arena -> 'x t -> 'x t -> bool
val bits : 'x kind -> int64 option

(** The strings a node holds, which its runtime function takes before its inputs. *)
val strings : 'x kind -> string list

val binary : 'x type_arena -> 'x t -> bool
val unary : 'x type_arena -> 'x t -> bool
val write : 'x type_arena -> 'x t -> bool
val commutative : 'x type_arena -> 'x t -> bool
val holds : 'x type_arena -> 'x t -> int
val structural : 'x t -> bool

(** Running the node twice on the same inputs differs from running it once: it allocates, may read what it wrote, ends a
    lifetime, or reaches the host or a callee. *)
val unrepeatable : 'x type_arena -> 'x t -> bool

val array_access : 'x type_arena -> 'x t -> array_access option
val regions : 'x t -> 'x region list
val region_node : 'x t -> bool
val output_count : 'x type_arena -> 'x t -> int
val output_type : 'x type_arena -> 'x t -> int -> Type.t
val link_type : 'x type_arena -> 'x link -> Type.t

(** The type of the link the node's input 0 reads. *)
val operand_type : 'x type_arena -> 'x t -> Type.t

(** The operand's type where the node takes it, else [otherwise], a type the node takes, so that verification rejects
    the operand. *)
val admitted : 'x type_arena -> 'x t -> (Type.t -> bool) -> Type.t -> Type.t

val output_types : 'x type_arena -> 'x t -> Type.t list
val outputs : 'x type_arena -> 'x t -> 'x link array
val operation : 'x t -> bool
val input_type : 'x type_arena -> 'x t -> int -> Type.t

(** Whether the node passes parts of its inputs through itself or its regions unchanged, as [flows] gives them; an
    output no flow reaches is computed by the node. *)
val forwarding : 'x type_arena -> 'x t -> bool

val flows : 'x type_arena -> 'x t -> ('x flow -> unit) -> unit
val reads : 'x type_arena -> 'x t -> bool
