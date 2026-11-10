open IR
open Definitions

let optimization =
  Options.optimization "invert-repeats" Control Restructuring 2
    "Test a loop's condition last, running its head once before the loop."

type 'x candidate = { matcher : 'x node; selector : 'x node; step : 'x node }

let two_variants node = match node.kind with SumFromI32 { variants; _ } -> List.length variants = 2 | _ -> false

let controlling step node =
  match node.kind with
  | RegionMatch branches when Array.length branches = 2 ->
      let selector = (RegionMatch.selector node).node in
      if two_variants selector && Link.equal (SumFromI32.tag step) (SumFromI32.tag selector) then
        Some { matcher = node; selector; step }
      else None
  | _ -> None

let candidate repeat =
  let step = (RegionRepeat.step repeat).node in
  if two_variants step then Array.find_map (controlling step) (RegionNodes.ordered (RegionRepeat.body repeat)) else None

let tail body matcher =
  let found = NodeSet.of_list [ matcher ] in
  Array.iter
    (fun node -> if Node.exists (fun input -> NodeSet.mem found input.node) node then ignore (NodeSet.add found node))
    (RegionNodes.ordered body);
  found

let head_values body candidate tail =
  Array.to_list (SumFromI32.values candidate.selector)
  @ List.concat_map
      (fun node ->
        if node == candidate.matcher then []
        else if node == candidate.step then Array.to_list (SumFromI32.values candidate.step)
        else Array.to_list node.inputs)
      (List.filter (NodeSet.mem tail) (Array.to_list (RegionNodes.ordered body)))
  |> List.filter (fun link -> not (NodeSet.mem tail link.node || Constant.is link.node))
  |> Link.distinct

let invert types repeat candidate =
  let body = RegionRepeat.body repeat in
  let values = head_values body candidate (tail body candidate.matcher) in
  let positions = LinkTable.create 16 in
  Array.iteri (fun position value -> LinkTable.replace positions value position) values;
  let value_types = Link.types types values in
  let choice = TypeArena.intern types (Sum [ value_types; value_types ]) in
  let head_of link held = Option.map (Array.get held) (LinkTable.find_opt positions link) in
  let head carried =
    let parameter link = if link.node == body.arguments then Some carried.(link.port) else None in
    let copied = Copying.links parameter (Array.append [| SumFromI32.tag candidate.selector |] values) in
    (copied.(0), Array.sub copied 1 (Array.length copied - 1))
  in
  let after variant held =
    let fields = Copying.links (fun link -> head_of link held) (SumFromI32.values candidate.selector) in
    let outputs = Copying.results (RegionMatch.branch candidate.matcher variant) fields in
    let read link = if link.node == candidate.matcher then Some outputs.(link.port) else head_of link held in
    Copying.links read (SumFromI32.values candidate.step)
  in
  let loop held =
    RegionRepeat.create types held value_types (fun carried ->
        let tag, next = head (after 1 carried) in
        Link.output (SumFromI32.create types choice tag next) 0)
  in
  let tag, first = head repeat.inputs in
  let skipped =
    RegionMatch.create types
      (Link.output (SumFromI32.create types choice tag first) 0)
      value_types
      (fun variant held ->
        if variant = 1 then
          let inverted = loop held in
          Array.init (Array.length held) (Link.output inverted)
        else held)
  in
  after 0 (Array.init (Array.length values) (Link.output skipped))

let inversion types node =
  match node.kind with RegionRepeat _ -> Option.map (invert types node) (candidate node) | _ -> None

let run (table : 'x Readers.table) = Rewrite.nodes (inversion table.region.types) table.nodes
