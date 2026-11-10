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

let domains = [ Integer; Number; Conversion; Sum; Storage; Control; Function; Graph ]

let domain_name domain =
  match domain with
  | Integer -> "integer"
  | Number -> "number"
  | Conversion -> "conversion"
  | Sum -> "sum"
  | Storage -> "storage"
  | Control -> "control"
  | Function -> "function"
  | Graph -> "graph"

let intent_name intent =
  match intent with
  | Folding -> "folding"
  | Identities -> "identities"
  | Comparisons -> "comparisons"
  | Forwarding -> "forwarding"
  | Narrowing -> "narrowing"
  | Restructuring -> "restructuring"
  | Calls -> "calls"
  | Sharing -> "sharing"
  | Boundaries -> "boundaries"
  | Ordering -> "ordering"
  | Checks -> "checks"

let highest = 3
let optimization name domain intent level summary = { name; domain; intent; level; summary }

let resolve level settings =
  let latest = List.rev settings in
  fun optimization ->
    List.find_map
      (fun selector -> List.assoc_opt selector latest)
      [ Optimization optimization.name; Intent (optimization.domain, optimization.intent); Domain optimization.domain ]
    |> Option.value ~default:(optimization.level <= level)

let all _ = true
let applied options optimization rewrite = if options optimization then rewrite () else None
