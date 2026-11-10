(** ArrowCrab places a node in a region when its identifier lies between the region's arguments and results, so a node's
    regions are numbered right after it. *)

open IR

val print : 'x CompilationUnit.t -> string
