open IR
open Definitions

val optimization : Options.optimization
val run : 'x region -> 'x region list -> 'x RegionSet.t
