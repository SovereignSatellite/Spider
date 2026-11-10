type t = Equal | LessThan | LessThanEqual

let commutative operator = match operator with Equal -> true | LessThan | LessThanEqual -> false

let holds operator =
  match operator with
  | Equal -> Outcomes.equal
  | LessThan -> Outcomes.less
  | LessThanEqual -> Outcomes.less lor Outcomes.equal

let name operator = match operator with Equal -> "Equal" | LessThan -> "LessThan" | LessThanEqual -> "LessThanEqual"
