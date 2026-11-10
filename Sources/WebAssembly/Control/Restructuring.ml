let rec patch graph exit seen id =
  let successors = (Graph.block graph id).successors in
  if successors = [] then Graph.add_edge graph id exit;
  List.iter (patch graph exit seen) (List.rev (List.filter (Bits.add seen) successors))

let run graph =
  let graph = Graph.compact graph in
  let exit = Graph.no_operation graph in
  let loops = Loops.run graph exit in
  patch graph exit (Bits.of_list [ 0; exit ]) 0;
  List.iter (fun (entry, latch) -> Graph.replace_edge graph latch entry latch) loops;
  Branches.run graph exit;
  List.iter (fun (entry, latch) -> Graph.replace_edge graph latch latch entry) loops;
  Graph.compact graph
