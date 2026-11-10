open LiveRanges

type interval = { congruence : int; leaf : IR.Type.t; mutable ranges : segment; mutable register : Register.t option }

let start interval = (get interval.ranges 0).from
let end_ interval = (get interval.ranges (interval.ranges.count - 1)).to_

let build ports flows liveness =
  let live = Array.of_list (List.filter (Liveness.live liveness) (List.init (Ports.count ports) Fun.id)) in
  let starts, grouped =
    Groups.by_key (Ports.count ports) (Flows.congruence flows)
      (fun value ->
        {
          from = Liveness.definition liveness value;
          to_ = Liveness.last_use liveness value;
          value = Flows.value_of flows value;
        })
      live
  in
  Array.init (Ports.count ports) (fun congruence ->
      match starts.(congruence + 1) - starts.(congruence) with
      | 0 -> None
      | count ->
          Some
            {
              congruence;
              leaf = Ports.leaf ports congruence;
              ranges = merge_overlapping (Array.sub grouped starts.(congruence) count);
              register = None;
            })
