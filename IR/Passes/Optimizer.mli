open IR
open Definitions

val settle : Options.t -> ('x type_arena -> 'x Readers.table -> 'x Local.rule list) -> 'x CompilationUnit.t -> unit

(** Merging waits until inlining finds nothing, since a merged body is named more often and may then not be inlined;
    neither adds a body, so [functions] still holds every body. *)
val run : Options.t -> 'x CompilationUnit.t -> unit
