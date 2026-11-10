(** Each leaf type is that of the value its register holds just before the move. *)
type 'x t =
  | Copy of Register.t * IR.Type.t * Register.t
  | Swap of Register.t * IR.Type.t * Register.t * IR.Type.t
  | Compute of 'x IR.Link.t * int * Register.t
