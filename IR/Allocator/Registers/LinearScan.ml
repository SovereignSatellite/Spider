open LiveRanges
open Intervals

let next_intersection (holder : Occupancy.holder) current =
  let ranges = holder.interval.ranges in
  let rec search holder_index current_index =
    if
      holder_index = ranges.count || current_index = current.ranges.count
      || (get ranges holder_index).from >= end_ current
    then None
    else
      let holder_range = get ranges holder_index in
      let current_range = get current.ranges current_index in
      if holder_range.to_ <= current_range.from then search (holder_index + 1) current_index
      else if current_range.to_ <= holder_range.from then search holder_index (current_index + 1)
      else Some (max holder_range.from current_range.from)
  in
  search holder.range 0

let by_start first second =
  match compare (start first) (start second) with 0 -> compare first.congruence second.congruence | order -> order

let lower_free free_until occupancy current =
  Occupancy.iter_inactive
    (fun holder ->
      match next_intersection holder current with
      | Some intersection ->
          Dynarray.set free_until holder.register (min (Dynarray.get free_until holder.register) intersection)
      | None -> ())
    occupancy

let holds_class classes register class_ = match classes.(register) with Some held -> held = class_ | None -> false

let free_register classes free_until occupancy current current_class =
  let rank register = (holds_class classes register current_class, Dynarray.get free_until register) in
  let chosen = ref None in
  for register = 0 to Occupancy.count occupancy - 1 do
    if
      (holds_class classes register current_class || Option.is_none classes.(register))
      && Dynarray.get free_until register >= end_ current
      && (not (Occupancy.is_active occupancy register))
      && Option.fold ~none:true ~some:(fun best -> rank register > rank best) !chosen
    then chosen := Some register
  done;
  !chosen

let replaceable_holder classes free_until occupancy current current_class =
  let ranked (holder : Occupancy.holder) = (end_ holder.interval, start holder.interval, holder.interval.congruence) in
  let chosen = ref None in
  for register = 0 to Occupancy.count occupancy - 1 do
    match Occupancy.active occupancy register with
    | Some holder
      when holds_class classes register current_class
           && end_ current <= Dynarray.get free_until register
           && end_ holder.interval > end_ current
           && Option.fold ~none:true ~some:(fun best -> ranked holder > ranked best) !chosen ->
        chosen := Some holder
    | _ -> ()
  done;
  !chosen

let assign_fixed (file : RegisterFile.t) intervals =
  let classes = Array.make file.registers None in
  let occupancy = Occupancy.create file.registers in
  let free_until = Dynarray.make file.registers 0 in
  let spilled = Dynarray.create () in
  let take current register =
    classes.(register) <- Some (file.class_ current.leaf);
    current.register <- Some (Fixed register);
    Occupancy.take occupancy current register
  in
  let replace current (holder : Occupancy.holder) =
    Dynarray.add_last spilled holder.interval;
    Occupancy.remove occupancy holder;
    take current holder.register
  in
  let replace_or_spill current =
    match replaceable_holder classes free_until occupancy current (file.class_ current.leaf) with
    | Some holder -> replace current holder
    | None -> Dynarray.add_last spilled current
  in
  let sorted = Array.of_list intervals in
  Array.stable_sort by_start sorted;
  Array.iter
    (fun current ->
      Occupancy.advance occupancy (start current);
      for register = 0 to Dynarray.length free_until - 1 do
        Dynarray.set free_until register max_int
      done;
      lower_free free_until occupancy current;
      match free_register classes free_until occupancy current (file.class_ current.leaf) with
      | Some register -> take current register
      | None -> replace_or_spill current)
    sorted;
  Dynarray.to_list spilled

let assign_pool first intervals =
  let occupancy = Occupancy.create 0 in
  let free_until = Dynarray.create () in
  let choose current =
    lower_free free_until occupancy current;
    let chosen = ref (Occupancy.unheld occupancy) in
    let rank slot = (Dynarray.get free_until slot, -slot) in
    Occupancy.iter_inactive
      (fun holder ->
        let slot = holder.register in
        if
          Dynarray.get free_until slot >= end_ current
          && (not (Occupancy.is_active occupancy slot))
          && Option.fold ~none:true ~some:(fun best -> rank slot > rank best) !chosen
        then chosen := Some slot)
      occupancy;
    Occupancy.iter_inactive (fun holder -> Dynarray.set free_until holder.register max_int) occupancy;
    match !chosen with
    | Some slot -> slot
    | None ->
        Dynarray.add_last free_until max_int;
        Occupancy.add_register occupancy
  in
  let sorted = Array.of_list intervals in
  Array.stable_sort by_start sorted;
  Array.iter
    (fun current ->
      Occupancy.advance occupancy (start current);
      let slot = choose current in
      current.register <- Some (Overflow (first + slot));
      Occupancy.take occupancy current slot)
    sorted;
  first + Occupancy.count occupancy

let group_by key items =
  let keys = Dynarray.create () in
  let groups = Hashtbl.create 8 in
  List.iter
    (fun item ->
      let key = key item in
      match Hashtbl.find_opt groups key with
      | Some group -> Hashtbl.replace groups key (item :: group)
      | None ->
          Dynarray.add_last keys key;
          Hashtbl.add groups key [ item ])
    items;
  List.map (fun key -> List.rev (Hashtbl.find groups key)) (Dynarray.to_list keys)

let assign (file : RegisterFile.t) intervals =
  ignore
    (List.fold_left assign_pool 0 (group_by (fun interval -> file.class_ interval.leaf) (assign_fixed file intervals)))
