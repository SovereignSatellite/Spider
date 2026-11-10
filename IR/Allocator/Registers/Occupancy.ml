open LiveRanges
open Intervals
module Ids = Hashtbl.Make (Int)
module Registers = Set.Make (Int)

type holder = { holder_id : int; interval : interval; register : int; mutable range : int; mutable live : bool }

module Changes = Set.Make (struct
  type t = int * holder

  let compare (due, holder) (other_due, other) = compare (due, holder.holder_id) (other_due, other.holder_id)
end)

type t = {
  mutable changes : Changes.t;
  active : holder option Dynarray.t;
  inactive : holder Ids.t;
  live_counts : int Dynarray.t;
  mutable unheld : Registers.t;
  mutable holders : int;
}

let create register_count =
  {
    changes = Changes.empty;
    active = Dynarray.make register_count None;
    inactive = Ids.create 16;
    live_counts = Dynarray.make register_count 0;
    unheld = Registers.of_list (List.init register_count Fun.id);
    holders = 0;
  }

let enqueue occupancy holder due = occupancy.changes <- Changes.add (due, holder) occupancy.changes

let release occupancy holder =
  holder.live <- false;
  let count = Dynarray.get occupancy.live_counts holder.register - 1 in
  Dynarray.set occupancy.live_counts holder.register count;
  if count = 0 then occupancy.unheld <- Registers.add holder.register occupancy.unheld

let activate occupancy holder =
  Ids.remove occupancy.inactive holder.holder_id;
  Dynarray.set occupancy.active holder.register (Some holder);
  enqueue occupancy holder (get holder.interval.ranges holder.range).to_

(** Another holder of the register may already be active from the position where this one stops. *)
let leave occupancy holder =
  match Dynarray.get occupancy.active holder.register with
  | Some current when current == holder -> Dynarray.set occupancy.active holder.register None
  | _ -> ()

let deactivate occupancy holder =
  leave occupancy holder;
  Ids.replace occupancy.inactive holder.holder_id holder;
  enqueue occupancy holder (get holder.interval.ranges holder.range).from

let expire occupancy holder =
  leave occupancy holder;
  Ids.remove occupancy.inactive holder.holder_id;
  release occupancy holder

let settle occupancy position holder =
  let ranges = holder.interval.ranges in
  let rec next range = if range < ranges.count && (get ranges range).to_ <= position then next (range + 1) else range in
  holder.range <- next holder.range;
  if holder.range = ranges.count then expire occupancy holder
  else if (get ranges holder.range).from <= position then activate occupancy holder
  else deactivate occupancy holder

let rec advance occupancy position =
  match Changes.min_elt_opt occupancy.changes with
  | Some ((due, holder) as change) when due <= position ->
      occupancy.changes <- Changes.remove change occupancy.changes;
      if holder.live then settle occupancy position holder;
      advance occupancy position
  | _ -> ()

let count occupancy = Dynarray.length occupancy.active
let active occupancy register = Dynarray.get occupancy.active register
let is_active occupancy register = Option.is_some (active occupancy register)
let iter_inactive visit occupancy = Ids.iter (fun _ holder -> visit holder) occupancy.inactive
let unheld occupancy = Registers.min_elt_opt occupancy.unheld

let add_register occupancy =
  Dynarray.add_last occupancy.active None;
  Dynarray.add_last occupancy.live_counts 0;
  Dynarray.length occupancy.active - 1

let take occupancy interval register =
  let holder = { holder_id = occupancy.holders; interval; register; range = 0; live = true } in
  occupancy.holders <- occupancy.holders + 1;
  Dynarray.set occupancy.live_counts register (Dynarray.get occupancy.live_counts register + 1);
  occupancy.unheld <- Registers.remove register occupancy.unheld;
  activate occupancy holder

let remove occupancy holder =
  Dynarray.set occupancy.active holder.register None;
  release occupancy holder
