type t = Equal | LessThan | LessThanEqual

val commutative : t -> bool
val holds : t -> int
val name : t -> string
