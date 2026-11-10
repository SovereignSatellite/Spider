open IR
open Definitions

let unchanged node outputs =
  Indices.for_all (Array.length outputs) (fun port -> Link.equal outputs.(port) (Link.output node port))

let nodes rule ordered =
  let replacements = NodeTable.create 16 in
  let changed = ref false in
  Array.iter
    (fun node ->
      Array.iteri
        (fun index input ->
          match NodeTable.find_opt replacements input.node with
          | Some outputs when not (Link.equal outputs.(input.port) input) ->
              node.inputs.(index) <- outputs.(input.port);
              changed := true
          | _ -> ())
        node.inputs;
      match rule node with
      | Some outputs when not (unchanged node outputs) -> NodeTable.replace replacements node outputs
      | _ -> ())
    ordered;
  !changed

let replace region node outputs =
  ignore (nodes (fun visited -> if visited == node then Some outputs else None) (RegionNodes.ordered region))

let substitute region replacements =
  let replaced = LinkTable.create 16 in
  List.iter (fun (link, replacement) -> LinkTable.replace replaced link replacement) replacements;
  Array.iter
    (fun node ->
      Array.iteri
        (fun index input ->
          match LinkTable.find_opt replaced input with
          | Some replacement -> node.inputs.(index) <- replacement
          | None -> ())
        node.inputs)
    (RegionNodes.ordered region)

let splice region substitutes =
  replace region region.arguments substitutes;
  Array.copy region.results.inputs

let rec nested rule changed outer =
  let ordered = RegionNodes.ordered outer in
  let inner =
    Array.fold_left
      (fun inner node ->
        List.fold_left (fun inner within -> nested rule changed within || inner) inner (Node.regions node))
      false ordered
  in
  if nodes rule ordered || inner then begin
    ignore (RegionSet.add changed outer);
    true
  end
  else false
