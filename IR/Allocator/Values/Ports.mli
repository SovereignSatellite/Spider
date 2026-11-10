open IR

type 'x t

val create : 'x Layouts.t -> 'x Schedule.t -> 'x t
val count : 'x t -> int
val port_count : 'x t -> int
val leaf : 'x t -> int -> Type.t
val port : 'x t -> int -> int -> int
val link : 'x t -> int -> 'x Link.t
val port_of : 'x t -> int -> int
val node_of : 'x t -> int -> int
val input_count : 'x t -> int -> int
val input_port : 'x t -> int -> int -> int
val port_of_link : 'x t -> 'x Link.t -> int
val start : 'x t -> int -> int
val end_ : 'x t -> int -> int
val node_start : 'x t -> int -> int
val node_end : 'x t -> int -> int
val value : 'x t -> 'x Link.t -> int -> int
