open IR
open Definitions

type 'x reads = { mutable last : ('x, int) LinkTable.t; readers : ('x node * int) Dynarray.t; earlier : int Dynarray.t }

let fresh_reads () = { last = LinkTable.create 16; readers = Dynarray.create (); earlier = Dynarray.create () }

let fill reads nodes =
  reads.last <- LinkTable.create (Array.length nodes);
  Dynarray.clear reads.readers;
  Dynarray.clear reads.earlier;
  Array.iter
    (fun node ->
      Array.iteri
        (fun index input ->
          Dynarray.add_last reads.earlier (Option.value (LinkTable.find_opt reads.last input) ~default:(-1));
          LinkTable.replace reads.last input (Dynarray.length reads.readers);
          Dynarray.add_last reads.readers (node, index))
        node.inputs)
    nodes;
  reads

let contains reads link = LinkTable.mem reads.last link

let of_reads reads link =
  let rec gather entry found =
    if entry < 0 then found else gather (Dynarray.get reads.earlier entry) (Dynarray.get reads.readers entry :: found)
  in
  match LinkTable.find_opt reads.last link with Some entry -> gather entry [] | None -> []

type 'x settling = {
  owner : 'x node option;
  inside : ('x, 'x reads) RegionTable.t;
  edited : 'x region Dynarray.t;
  visited : 'x NodeSet.t;
  order : 'x node Dynarray.t;
  own : 'x reads;
}

let settling owner =
  {
    owner;
    inside = RegionTable.create 16;
    edited = Dynarray.create ();
    visited = NodeSet.create 64;
    order = Dynarray.create ();
    own = fresh_reads ();
  }

let edits settling = settling.edited

type 'x table = { region : 'x region; settling : 'x settling; nodes : 'x node array; own_reads : 'x reads Lazy.t }

let create ?(settling = settling None) region =
  let nodes = RegionNodes.ordered_with settling.visited settling.order region in
  { region; settling; nodes; own_reads = lazy (fill settling.own nodes) }

let owner table = table.settling.owner
let renewed table = create ~settling:table.settling table.region

let edited table within =
  RegionTable.remove table.settling.inside within;
  Dynarray.add_last table.settling.edited within

let redefine_match table matcher selector output_types build =
  RegionMatch.redefine table.region.types matcher selector output_types build;
  Array.iter (edited table) (RegionMatch.branches matcher)

let redefine_repeat table repeat initial exit_types build =
  RegionRepeat.redefine table.region.types repeat initial exit_types build;
  edited table (RegionRepeat.body repeat)

let in_ table within =
  if within == table.region then Lazy.force table.own_reads
  else
    match RegionTable.find_opt table.settling.inside within with
    | Some reads -> reads
    | None ->
        let reads = fill (fresh_reads ()) (RegionNodes.ordered within) in
        RegionTable.replace table.settling.inside within reads;
        reads

let of_link readers region link = of_reads (in_ readers region) link
let is_read readers region link = contains (in_ readers region) link

let only_by readers region reader node =
  Indices.for_all (Node.output_count region.types node) (fun port ->
      List.for_all (fun (other, _) -> other == reader) (of_link readers region (Link.output node port)))
