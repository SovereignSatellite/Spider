type t = Equal | LessThanS | LessThanU

val commutative : t -> bool
val holds : t -> int
val name : t -> string
