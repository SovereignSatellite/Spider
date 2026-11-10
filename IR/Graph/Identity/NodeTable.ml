open Definitions
module Ids = Hashtbl.Make (Int)

type ('x, 'value) t = 'value Ids.t

let create size = Ids.create size
let find_opt table node = Ids.find_opt table node.id
let find table node = Ids.find table node.id
let mem table node = Ids.mem table node.id
let replace table node value = Ids.replace table node.id value
let length table = Ids.length table
