open Plan

type outcome = Returned of int64 list | Trapped | Uninstantiated
type run = Finished of string | Stopped of string * string
type target = { name : string; write : string -> plan -> unit; run : string -> run }

val lead : unit -> unit
val interrupted : int -> unit
val run : string -> string -> string list -> run
val outcomes : string -> (int, outcome) Hashtbl.t
val distinct : 'item list -> 'item list
val callers : step list -> signature list * (signature -> string)
val readers : step list -> kind list * (kind -> string)
val instances : step list -> instance list
val write_lines : string -> string list -> unit
val shown : kind -> (int -> string) -> string
