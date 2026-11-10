type t = Add | Subtract | Multiply | Divide | Minimum | Maximum | CopySign

val commutative : t -> bool
val name : t -> string
