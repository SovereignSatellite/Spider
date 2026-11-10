open IR
open Definitions

type 'x t = { path : ('x node * int) list; nodes : 'x node array; outer : ('x, 'x link) LinkTable.t }

let outer_links matcher branch values outer_of =
  let outer = LinkTable.create 16 in
  Array.iteri
    (fun port value ->
      if not (Branch.knows (RegionMatch.selector matcher) value) then
        Option.iter (LinkTable.replace outer (Region.parameter branch port)) (outer_of value))
    values;
  outer

let rec enter visit path outer_of nodes =
  Array.iter
    (fun node ->
      match node.kind with
      | RegionMatch branches ->
          Option.iter
            (fun (_, values) -> Array.iteri (enter_branch visit path outer_of node values) branches)
            (Branch.uniform node)
      | _ -> ())
    nodes

and enter_branch visit path outer_of matcher values variant branch =
  let outer = outer_links matcher branch values outer_of in
  if LinkTable.length outer > 0 then begin
    let nested = { path = path @ [ (matcher, variant) ]; nodes = RegionNodes.ordered branch; outer } in
    visit nested;
    enter visit nested.path (LinkTable.find_opt outer) nested.nodes
  end

let visit nodes visit = enter visit [] Option.some nodes
