type t = private int

val of_scalar : [< Scalar.t ] -> t
val scalar : t -> Scalar.t option
val i8 : t
val i16 : t
val i32 : t
val i64 : t
val f32 : t
val f64 : t
val unit : t

(** Two empty variants: the first is false and the second true. *)
val boolean : t

val bytes : t
val opaque : t
val bits : t -> int
val numbered : int -> t
