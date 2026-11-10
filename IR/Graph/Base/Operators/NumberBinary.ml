type t = Add | Subtract | Multiply | Divide | Minimum | Maximum | CopySign

let commutative operator =
  match operator with Add | Multiply | Minimum | Maximum -> true | Subtract | Divide | CopySign -> false

let name operator =
  match operator with
  | Add -> "Add"
  | Subtract -> "Subtract"
  | Multiply -> "Multiply"
  | Divide -> "Divide"
  | Minimum -> "Minimum"
  | Maximum -> "Maximum"
  | CopySign -> "CopySign"
