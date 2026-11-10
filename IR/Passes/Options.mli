type domain = Integer | Number | Conversion | Sum | Storage | Control | Function | Graph

type intent =
  | Folding
  | Identities
  | Comparisons
  | Forwarding
  | Narrowing
  | Restructuring
  | Calls
  | Sharing
  | Boundaries
  | Ordering
  | Checks

type optimization = { name : string; domain : domain; intent : intent; level : int; summary : string }
type selector = Domain of domain | Intent of domain * intent | Optimization of string
type t = optimization -> bool

val domains : domain list
val domain_name : domain -> string
val intent_name : intent -> string
val highest : int
val optimization : string -> domain -> intent -> int -> string -> optimization
val resolve : int -> (selector * bool) list -> t
val all : t
val applied : t -> optimization -> (unit -> 'result option) -> 'result option
