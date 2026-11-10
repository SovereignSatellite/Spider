open IR
open Definitions

let word bits index = if index < Array.length bits then bits.(index) else 0

let union left right =
  if Indices.for_all (Array.length right) (fun index -> right.(index) land lnot (word left index) = 0) then left
  else Array.init (max (Array.length left) (Array.length right)) (fun index -> word left index lor word right index)

let bit index = 1 lsl (index mod Sys.int_size)
let holds bits index = word bits (index / Sys.int_size) land bit index <> 0

type 'x t = { types : 'x type_arena; writers : 'x node Dynarray.t; cones : ('x, int array) NodeTable.t }

let create types = { types; writers = Dynarray.create (); cones = NodeTable.create 256 }

let alone writers node =
  let index = Dynarray.length writers.writers in
  Dynarray.add_last writers.writers node;
  Array.init ((index / Sys.int_size) + 1) (fun position -> if position = index / Sys.int_size then bit index else 0)

let rec build_cone writers node =
  let inputs = Array.fold_left (fun inputs input -> union inputs (cone writers input.node)) [||] node.inputs in
  let cone = if Accesses.writes_anything writers.types node then union inputs (alone writers node) else inputs in
  NodeTable.replace writers.cones node cone;
  cone

and cone writers node =
  match NodeTable.find_opt writers.cones node with Some cone -> cone | None -> build_cone writers node

let besides writers links node =
  let after = List.fold_left (fun cone_of link -> union cone_of (cone writers link.node)) [||] links in
  let before = cone writers node in
  List.filter_map
    (fun index ->
      if holds after index && not (holds before index) then Some (Dynarray.get writers.writers index) else None)
    (List.init (Array.length after * Sys.int_size) Fun.id)
