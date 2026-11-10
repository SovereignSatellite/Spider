val write_name : Buffer.t -> Name.t -> unit
val write_items : Buffer.t -> ('item -> bool) -> ('item -> unit) -> 'item list -> unit
val separated : Buffer.t -> string -> ('item -> unit) -> 'item list -> unit
val enclosed : Buffer.t -> string -> string -> ('item -> unit) -> 'item list -> unit
val write_unsigned : Buffer.t -> int64 -> unit
val write_source : Buffer.t -> string -> unit
