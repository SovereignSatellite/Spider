(** The bytes as a string literal holds them, without the quotes: printable ASCII as is but for the quote and the
    backslash, every other byte as [\xNN]. Only printable ASCII is written, so the text cannot end a comment. *)
val write_unquoted : Buffer.t -> string -> unit

val write : Buffer.t -> string -> unit
