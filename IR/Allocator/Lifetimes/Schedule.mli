open IR

type 'x t

val create : 'x Region.t -> 'x t
val body : 'x t -> 'x Region.t
val count : 'x t -> int
val node : 'x t -> int -> 'x Node.t
val number : 'x t -> 'x Node.t -> int
val start : 'x t -> 'x Region.t -> int
val end_ : 'x t -> 'x Region.t -> int
