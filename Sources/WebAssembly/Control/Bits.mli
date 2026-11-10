type t

val create : unit -> t
val contains : t -> int -> bool
val add : t -> int -> bool
val remove : t -> int -> unit
val add_all : t -> int array -> unit
val clear : t -> unit
val copy : t -> t
val to_array : t -> int array
val of_list : int list -> t
