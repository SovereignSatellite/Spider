open Definitions
module Keys = Hashtbl.Make (Int)

type ('x, 'value) t = 'value Keys.t

let key link = (link.node.id lsl 16) lor link.port
let create size = Keys.create size
let find_opt table link = Keys.find_opt table (key link)
let find table link = Keys.find table (key link)
let mem table link = Keys.mem table (key link)
let replace table link value = Keys.replace table (key link) value
let length table = Keys.length table
