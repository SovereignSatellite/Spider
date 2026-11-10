open IR

type 'x t

val create : unit -> 'x t
val name : 'x t -> 'x Region.t -> Name.t
val build : 'x t -> ('x Region.t -> Name.t -> 'item) -> 'item list
