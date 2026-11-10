open IR
open Definitions

let optimization =
  Options.optimization "collapse-matches" Control Restructuring 2
    "Replace a Match whose possible branches are equal with that branch."

let possible matcher =
  match Sums.built (RegionMatch.selector matcher).node with
  | Some built -> List.map fst built
  | None -> List.init (Array.length (RegionMatch.branches matcher)) Fun.id

let splice_first types readers matcher first rest =
  let given = Branch.values matcher first in
  let same_value left_port right_port = left_port = right_port && Option.is_some given in
  if
    (Option.is_some given || not (Branch.reads_fields readers matcher first))
    && List.for_all
         (fun variant ->
           Equivalence.regions types same_value (RegionMatch.branch matcher first) (RegionMatch.branch matcher variant))
         rest
  then
    Some
      (Rewrite.splice (RegionMatch.branch matcher first)
         (match given with Some given -> given | None -> RegionMatch.fields matcher first))
  else None

let same types readers matcher =
  match possible matcher with first :: rest -> splice_first types readers matcher first rest | [] -> None

let rebuilds_variant types matcher variant result =
  match result.node.kind with
  | SumNew { variant = built; sum_type } ->
      built = variant
      && sum_type = Link.type_ types (RegionMatch.selector matcher)
      && Indices.for_all (Array.length result.node.inputs) (fun index ->
          Link.equal result.node.inputs.(index) (RegionMatch.field matcher variant index))
  | _ -> false

let holds_variant types variant result = Branch.variant_of types result = Some variant

let tag_as types value_type tag =
  if value_type = Type.i32 then tag else Link.output (SumFromI32.create types value_type tag [||]) 0

let rebuilt types matcher =
  let branches = RegionMatch.branches matcher in
  let every gives port =
    Indices.for_all (Array.length branches) (fun variant -> gives variant branches.(variant).results.inputs.(port))
  in
  let selector = RegionMatch.selector matcher in
  Array.init (Node.output_count types matcher) (fun port ->
      if every (rebuilds_variant types matcher) port then selector
      else if Array.length branches > 1 && every (holds_variant types) port then
        tag_as types (Node.output_type types matcher port) (Link.output (Node.create I32FromSum [| selector |]) 0)
      else match Relations.of_match types matcher port with Some link -> link | None -> Link.output matcher port)

let collapse types readers node =
  match node.kind with
  | RegionMatch _ -> (
      match same types readers node with Some _ as spliced -> spliced | None -> Some (rebuilt types node))
  | _ -> None

let run types table = Rewrite.nodes (collapse types table) table.nodes
