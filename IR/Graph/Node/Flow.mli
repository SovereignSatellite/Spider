open Definitions

type 'x t = 'x flow

val whole : 'x node -> int -> 'x link -> 'x t
val written : 'x node -> 'x t
