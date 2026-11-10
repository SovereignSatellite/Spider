open IR
open Definitions

type 'x rule = 'x node -> 'x link array option

let rules options types table =
  [ NodeReduction.reduce options types table; ConstantFolding.fold options types; Identities.fold options types table ]

let first rules node = List.find_map (fun rule -> rule node) rules
let run rules (table : 'x Readers.table) = Rewrite.nodes (first (rules table)) table.nodes
