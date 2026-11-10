open IR
open Definitions

let optimization =
  Options.optimization "reduce-repeats" Control Restructuring 2
    "Replace a loop whose step never continues with its body."

let splice_once types readers repeat =
  let body = RegionRepeat.body repeat in
  let step = (Rewrite.splice body repeat.inputs).(0) in
  let fields sum _ = Option.get (Sums.fields sum 0) in
  Some (Step.rebuild types readers (Node.output_types types repeat) fields [||] body step)

let reduce_repeat types readers repeat =
  match Step.producers readers (RegionRepeat.body repeat) (RegionRepeat.step repeat) with
  | Some producers when List.is_empty (Step.building 1 producers) -> splice_once types readers repeat
  | _ -> None

let reduce types readers node = match node.kind with RegionRepeat _ -> reduce_repeat types readers node | _ -> None
let run (table : 'x Readers.table) = Rewrite.nodes (reduce table.region.types table) table.nodes
