(** The JSON the suite holds: no whitespace between tokens, line numbers as its only bare values, and [\u] escapes only
    of control characters. *)
type t = Number of int | String of string | List of t list | Object of (string * t) list

val parse : string -> t
val property : t -> string -> t option
val get : t -> string -> t
val text : t -> string
