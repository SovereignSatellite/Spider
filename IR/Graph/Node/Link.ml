open Definitions

type 'x t = 'x link

let output node index =
  if index > 0xFFFF then invalid_arg "A port is below 65,536.";
  { node; port = index }

let type_ = Node.link_type
let types types links = List.map (type_ types) (Array.to_list links)
let equal left right = left.node == right.node && left.port = right.port

let distinct links =
  let seen = LinkTable.create 16 in
  Array.of_list
    (List.filter
       (fun link ->
         (not (LinkTable.mem seen link))
         &&
         (LinkTable.replace seen link ();
          true))
       links)

let index_of links link = Option.get (Array.find_index (equal link) links)
