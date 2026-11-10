open Definitions
module Ids = Hashtbl.Make (Int)

type ('x, 'value) t = 'value Ids.t

let create size = Ids.create size
let find_opt table region = Ids.find_opt table region.region_id
let find table region = Ids.find table region.region_id
let mem table region = Ids.mem table region.region_id
let replace table region value = Ids.replace table region.region_id value
let remove table region = Ids.remove table region.region_id
let length table = Ids.length table
