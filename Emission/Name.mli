type t =
  | Text of string
  | Numbered of string * int
  | Region of int
  | Register of Allocator.Register.t
  | Helper of string
  | Constant of string
