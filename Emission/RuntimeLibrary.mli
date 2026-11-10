type t
type runtime_library := t

val sources : t -> string list

module NeededHelpers : sig
  type t

  val create : runtime_library -> t
  val need : t -> string -> unit
  val sources : t -> string list
end

val of_resources : string -> (string * string) list -> t

(** The name of the runtime helper implementing the node. *)
val helper_name : 'x IR.Definitions.type_arena -> 'x IR.Node.t -> string
