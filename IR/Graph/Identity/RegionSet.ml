open Definitions
module Ids = Hashtbl.Make (Int)

type 'x t = unit Ids.t

let create size = Ids.create size
let mem set region = Ids.mem set region.region_id

let add set region =
  (not (Ids.mem set region.region_id))
  &&
  (Ids.replace set region.region_id ();
   true)

let remove set region = Ids.remove set region.region_id
let length set = Ids.length set
let union set other = Ids.iter (Ids.replace set) other
let except set other = Ids.iter (fun id () -> Ids.remove set id) other
