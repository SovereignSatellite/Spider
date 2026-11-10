type scratch

val scratch : unit -> scratch
val continuation_point : Graph.t -> scratch -> int -> int * int list list
