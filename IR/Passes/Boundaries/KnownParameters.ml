open IR
open Definitions

let optimization =
  Options.optimization "propagate-known-parameters" Graph Boundaries 2
    "Replace a region parameter given the same value on every path with that value."

let replace readers region known =
  let substitutes =
    Array.init (Array.length region.parameter_types) (fun index ->
        let parameter = Region.parameter region index in
        match known index with Some link when Readers.is_read readers region parameter -> link | _ -> parameter)
  in
  if
    Indices.exists (Array.length substitutes) (fun index ->
        not (Link.equal substitutes.(index) (Region.parameter region index)))
  then begin
    Rewrite.replace region region.arguments substitutes;
    Readers.edited readers region
  end

let in_branch types matcher variant values index =
  match
    (Branch.known types (RegionMatch.selector matcher) variant values.(index), Link.index_of values values.(index))
  with
  | None, earlier when earlier < index -> Some (Region.parameter (RegionMatch.branch matcher variant) earlier)
  | known, _ -> known

let replace_into types readers node =
  match node.kind with
  | RegionMatch branches ->
      Array.iteri
        (fun variant branch ->
          Option.iter
            (fun values -> replace readers branch (in_branch types node variant values))
            (Branch.values node variant))
        branches
  | RegionRepeat body -> replace readers body (KnownValue.carried_known types node)
  | _ -> ()

let run (table : 'x Readers.table) =
  Array.iter (replace_into table.region.types table) table.nodes;
  false
