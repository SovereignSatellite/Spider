open Definitions

let rec node node inputs =
  match node.kind with
  | RegionArguments _ | RegionResults _ -> invalid_arg "A region's boundary is copied only with its region."
  | RegionMatch branches -> Node.create (RegionMatch (Array.map region branches)) inputs
  | RegionRepeat body -> Node.create (RegionRepeat (region body)) inputs
  | Throw result_types -> Node.create (Throw (Dynarray.copy result_types)) inputs
  | kind -> Node.create kind inputs

and region region =
  Region.create
    ~build:(fun copied -> results region (Region.parameters copied))
    region.types (Region.parameter_types region) (Region.result_types region)

and links substitute links =
  let copies = NodeTable.create 64 in
  let rec copy_link link =
    match substitute link with Some substituted -> substituted | None -> { link with node = copy_held link.node }
  and copy_held held = match NodeTable.find_opt copies held with Some copied -> copied | None -> create_copy held
  and create_copy original =
    let copied = node original (Array.map copy_link original.inputs) in
    NodeTable.replace copies original copied;
    copied
  in
  Array.map copy_link links

and results region substitutes =
  let parameter link = if link.node == region.arguments then Some substitutes.(link.port) else None in
  links parameter region.results.inputs
