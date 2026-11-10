type t = Equal | LessThanS | LessThanU

let commutative operator = match operator with Equal -> true | LessThanS | LessThanU -> false
let holds operator = match operator with Equal -> Outcomes.equal | LessThanS | LessThanU -> Outcomes.less
let name operator = match operator with Equal -> "Equal" | LessThanS -> "LessThanS" | LessThanU -> "LessThanU"
