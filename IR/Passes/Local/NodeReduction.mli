open IR
open Definitions

val canonical_offsets : Options.optimization
val offset_comparisons : Options.optimization
val fences : Options.optimization
val throws : Options.optimization
val reduce : Options.t -> 'x type_arena -> 'x Readers.table -> 'x node -> 'x link array option
