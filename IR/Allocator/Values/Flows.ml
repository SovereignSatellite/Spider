open IR

type edge = { source : int; destination : int; copied : bool }
type 'item sole = Nothing | Sole of 'item | Several
type origin = Computed | Undefined | Joined | Copied

let adding item found =
  match found with Nothing -> Sole item | Sole held when held = item -> found | Sole _ | Several -> Several

type 'x t = {
  types : 'x TypeArena.t;
  schedule : 'x Schedule.t;
  ports : 'x Ports.t;
  congruences : Congruences.t;
  origins : origin array;
  values : Congruences.t;
  edge_starts : int array;
  grouped_edges : edge array;
}

let types flows = flows.types
let congruence flows value = Congruences.find flows.congruences value
let join flows first second = Congruences.join flows.congruences first second
let undefined flows value = flows.origins.(congruence flows value) = Undefined
let value_of flows value = Congruences.find flows.values (congruence flows value)
let reached flows value = match flows.origins.(value) with Joined | Copied -> true | Computed | Undefined -> false
let copied flows value = flows.origins.(value) = Copied

(** Whether the node reads nothing and every leaf of it from [first] on flows through it or is undefined. *)
let flows_from flows number first =
  let node = Schedule.node flows.schedule number in
  (not (Node.region_node node))
  && Node.forwarding flows.types node
  && (not (Node.reads flows.types node))
  &&
  let rec all value =
    value = Ports.node_end flows.ports number || ((reached flows value || undefined flows value) && all (value + 1))
  in
  all first

let computes_nothing flows number = flows_from flows number (Ports.node_start flows.ports number)

let computes_first flows number =
  let first = Ports.node_start flows.ports number in
  first < Ports.node_end flows.ports number
  && (not (reached flows first))
  && (not (undefined flows first))
  && flows_from flows number (first + 1)

let edges flows number =
  Array.sub flows.grouped_edges flows.edge_starts.(number) (flows.edge_starts.(number + 1) - flows.edge_starts.(number))

let connected (layouts : 'x Layouts.t) schedule ports =
  let types = Layouts.types layouts in
  let congruences = Congruences.create (Ports.count ports) in
  let origins = Array.make (Ports.count ports) Computed in
  let edges = Dynarray.create () in
  let readers = Array.make (Ports.port_count ports) Nothing in
  let collect (flow : 'x Flow.t) =
    let port = Ports.port_of_link ports flow.output in
    readers.(port) <- adding (Schedule.number schedule flow.reader) readers.(port)
  in
  let leave_slots_undefined number =
    for
      port = Ports.port ports number 0
      to Ports.port ports number (Node.output_count types (Schedule.node schedule number)) - 1
    do
      if Layouts.tagged layouts (Link.type_ types (Ports.link ports port)) then
        Array.fill origins (Ports.start ports port + 1) (Ports.end_ ports port - Ports.start ports port - 1) Undefined
    done
  in
  let connect (flow : 'x Flow.t) =
    let reader = Schedule.number schedule flow.reader in
    let source = flow.reader.inputs.(flow.input) in
    let source_start = Ports.start ports (Ports.port_of_link ports source) in
    let destination_start = Ports.start ports (Ports.port_of_link ports flow.output) in
    let sources = Layouts.select layouts (Link.type_ types source) flow.input_path in
    let destinations = Layouts.select layouts (Link.type_ types flow.output) flow.output_path in
    let copied = readers.(Ports.port_of_link ports flow.output) = Several in
    Array.iteri
      (fun leaf source_leaf ->
        let edge =
          { source = source_start + source_leaf; destination = destination_start + destinations.(leaf); copied }
        in
        origins.(edge.destination) <- (if copied then Copied else Joined);
        if not copied then Congruences.join congruences edge.source edge.destination;
        if copied || not flow.reads then Dynarray.add_last edges (reader, edge))
      sources
  in
  let forward number node =
    leave_slots_undefined number;
    Node.flows types node collect;
    Node.flows types node connect
  in
  for number = 0 to Schedule.count schedule - 1 do
    let node = Schedule.node schedule number in
    if Node.forwarding types node then forward number node
  done;
  (congruences, origins, Dynarray.to_array edges)

let create layouts schedule ports =
  let count = Ports.count ports in
  let congruences, origins, edges = connected layouts schedule ports in
  let values = Congruences.create count in
  let value_of value = Congruences.find values (Congruences.find congruences value) in
  let share_trivial source_starts sources destination =
    let own = value_of destination in
    let others = ref Nothing in
    for index = source_starts.(destination) to source_starts.(destination + 1) - 1 do
      let source = sources.(index) in
      let held = value_of source in
      if held <> own && origins.(Congruences.find congruences source) <> Undefined then others := adding held !others
    done;
    match !others with Sole other -> Congruences.join values other own | Nothing | Several -> ()
  in
  let copies = Array.of_seq (Seq.filter (fun (_, edge) -> edge.copied) (Array.to_seq edges)) in
  let source_starts, sources =
    Groups.by_key count (fun (_, edge) -> edge.destination) (fun (_, edge) -> edge.source) copies
  in
  (* The last destinations go first, since one in a nested region, numbered later, can be the source of one enclosing
     it. *)
  for destination = count - 1 downto 0 do
    share_trivial source_starts sources destination
  done;
  let edge_starts, grouped_edges = Groups.by_key (Schedule.count schedule) fst snd edges in
  { types = Layouts.types layouts; schedule; ports; congruences; origins; values; edge_starts; grouped_edges }
