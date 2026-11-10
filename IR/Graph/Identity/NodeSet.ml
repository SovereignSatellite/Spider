open Definitions
module Ids = Hashtbl.Make (Int)

type 'x t = unit Ids.t

let create size = Ids.create size
let mem set node = Ids.mem set node.id

let add set node =
  (not (Ids.mem set node.id))
  &&
  (Ids.replace set node.id ();
   true)

let remove set node = Ids.remove set node.id
let clear set = Ids.reset set

let of_list nodes =
  let set = create (List.length nodes) in
  List.iter (fun node -> ignore (add set node)) nodes;
  set
