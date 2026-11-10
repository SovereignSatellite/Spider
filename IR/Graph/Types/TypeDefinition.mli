(** A type an extension defines, interned structurally. *)
type extension = ..

type t = Structural of StructuralType.t | Box of Type.t | Extension of extension
