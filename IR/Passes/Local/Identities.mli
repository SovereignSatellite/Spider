open IR
open Definitions

val commutative_order : Options.optimization
val integer_arithmetic : Options.optimization
val extension_masks : Options.optimization
val tag_masks : Options.optimization
val constant_reassociation : Options.optimization
val narrowed_arithmetic : Options.optimization
val integer_comparisons : Options.optimization
val extension_comparisons : Options.optimization
val inverse_comparisons : Options.optimization
val number_arithmetic : Options.optimization
val cancelled_conversions : Options.optimization
val retypes : Options.optimization
val sums : Options.optimization
val unchanging_writes : Options.optimization
val relations : Options.optimization
val fold : Options.t -> 'x type_arena -> 'x Readers.table -> 'x node -> 'x link array option
