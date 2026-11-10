open IR

type 'x t = {
  schedule : 'x Schedule.t;
  ports : 'x Ports.t;
  flows : 'x Flows.t;
  liveness : 'x Liveness.t;
  intervals : Intervals.interval option array;
  held : Type.t list Register.Map.t;
}

let register_of allocation value =
  Option.bind allocation.intervals.(Flows.congruence allocation.flows value) (fun interval -> interval.register)

let evaluated allocation region =
  let schedule = allocation.schedule in
  List.filter_map
    (fun number -> if Liveness.apart allocation.liveness number then Some (Schedule.node schedule number) else None)
    (List.init
       (Schedule.end_ schedule region - Schedule.start schedule region)
       (fun offset -> Schedule.start schedule region + offset))

(** The inlined output computing the value where it is read, as its first leaf: through forwarding nodes, the one the
    value comes from. *)
let computing allocation value =
  let ports = allocation.ports in
  let source = Flows.congruence allocation.flows value in
  let port = Ports.port_of ports source in
  if Liveness.inlined allocation.liveness (Ports.node_of ports port) && source = Ports.start ports port then
    Some (Ports.link ports port)
  else None

let inlined allocation link leaf = computing allocation (Ports.value allocation.ports link leaf)
let deciding allocation region tested = Decisions.deciding allocation.schedule allocation.flows region tested
let following allocation repeat = Liveness.following allocation.liveness (Schedule.number allocation.schedule repeat)
let register allocation link leaf = register_of allocation (Ports.value allocation.ports link leaf)

let moves allocation reader =
  let ports = allocation.ports in
  let edges = Array.to_list (Flows.edges allocation.flows (Schedule.number allocation.schedule reader)) in
  ParallelCopy.sequentialize
    (List.filter_map
       (fun (edge : Flows.edge) ->
         match (edge.copied, register_of allocation edge.source, register_of allocation edge.destination) with
         | true, Some source, Some destination when source <> destination ->
             Some (source, destination, Ports.leaf ports edge.destination)
         | _ -> None)
       edges)
  @ List.filter_map
      (fun (edge : Flows.edge) ->
        match
          (edge.copied && Option.is_some (computing allocation edge.source), register_of allocation edge.destination)
        with
        | true, Some destination ->
            let port = Ports.port_of ports edge.source in
            Some (Move.Compute (Ports.link ports port, edge.source - Ports.start ports port, destination))
        | _ -> None)
      edges

let registers allocation = List.map fst (Register.Map.bindings allocation.held)
let leaf_types allocation register = Register.Map.find register allocation.held

let held_leaf_types intervals =
  let with_leaf_type leaf leaf_types =
    match leaf_types with
    | None -> Some [ leaf ]
    | Some leaf_types -> Some (if List.mem leaf leaf_types then leaf_types else leaf_types @ [ leaf ])
  in
  List.fold_left
    (fun held (interval : Intervals.interval) ->
      Register.Map.update (Option.get interval.register) (with_leaf_type interval.leaf) held)
    Register.Map.empty intervals

let allocate layouts file reads_once computed_where_read body =
  let schedule = Schedule.create body in
  let ports = Ports.create layouts schedule in
  let flows = Flows.create layouts schedule ports in
  let liveness = Liveness.create schedule ports flows reads_once computed_where_read in
  let intervals = Intervals.build ports flows liveness in
  Coalescing.coalesce schedule flows intervals;
  let live = List.filter_map Fun.id (Array.to_list intervals) in
  LinearScan.assign file live;
  { schedule; ports; flows; liveness; intervals; held = held_leaf_types live }
