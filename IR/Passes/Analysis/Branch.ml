open IR
open Definitions

let reads_fields readers matcher variant =
  let branch = RegionMatch.branch matcher variant in
  Indices.exists (Array.length branch.parameter_types) (fun index ->
      Readers.is_read readers branch (Region.parameter branch index))

let values matcher variant = Sums.fields (RegionMatch.selector matcher).node variant

let outside matcher variant link =
  if link.node == (RegionMatch.branch matcher variant).arguments then
    Sums.field (RegionMatch.selector matcher).node variant link.port
  else None

let rec root link = match link.node.kind with I32FromSum | SumFromI32 _ -> root link.node.inputs.(0) | _ -> link

let variant types value_type variant =
  match TypeArena.definition types value_type with
  | _ when value_type = Type.i32 -> Some (Link.output (Node.create (I32Constant variant) [||]) 0)
  | Structural (Sum variants) when List.nth variants variant = [] ->
      Some (Link.output (Node.create (SumNew { sum_type = value_type; variant }) [||]) 0)
  | _ -> None

let variant_of types link =
  match link.node.kind with
  | SumNew { variant; _ } when Array.length link.node.inputs = 0 -> Some variant
  | _ when Link.type_ types link = Type.i32 ->
      Option.map (fun bits -> Int32.to_int (Int64.to_int32 bits)) (Constant.bits link)
  | _ -> None

let knows selector link = Constant.is link.node || Link.equal (root link) (root selector)

let known types selector branch_variant link =
  if Constant.is link.node then Some (Constant.copy link)
  else if Link.equal (root link) (root selector) then variant types (Link.type_ types link) branch_variant
  else None

let tagged types variants tag values =
  let sum_type = TypeArena.intern types (Sum (List.init variants (fun _ -> Link.types types values))) in
  Link.output (SumFromI32.create types sum_type tag values) 0

let choose types condition values output_types on_false on_true =
  let selector =
    if Array.length values = 0 then condition
    else tagged types 2 (Link.output (Node.create I32FromSum [| condition |]) 0) values
  in
  let matcher =
    RegionMatch.create types selector output_types (fun variant fields ->
        (if variant = 0 then on_false else on_true) fields)
  in
  Array.init (List.length output_types) (Link.output matcher)

let choose_one types condition values output_type on_false on_true =
  (choose types condition values [ output_type ]
     (fun fields -> [| on_false fields |])
     (fun fields -> [| on_true fields |])).(0)

let both types first second =
  choose_one types first [| second |] Type.boolean (fun _ -> Constant.boolean false) (fun fields -> fields.(0))

let either types first second =
  choose_one types first [| second |] Type.boolean (fun fields -> fields.(0)) (fun _ -> Constant.boolean true)

let uniform matcher =
  let selector = RegionMatch.selector matcher in
  let branches = RegionMatch.branches matcher in
  match selector.node.kind with
  | SumFromI32 { variants; _ } when List.length variants = Array.length branches ->
      Some (SumFromI32.tag selector.node, SumFromI32.values selector.node)
  | _ when Array.length branches > 1 && Array.for_all (fun branch -> Array.length branch.parameter_types = 0) branches
    ->
      Some (Link.output (Node.create I32FromSum [| selector |]) 0, [||])
  | _ -> None

let with_values types matcher variants values =
  let sum = (RegionMatch.selector matcher).node in
  let fields variant branch =
    if List.mem variant variants then Link.types types values else Region.parameter_types branch
  in
  let sum_type = TypeArena.intern types (Sum (List.mapi fields (Array.to_list (RegionMatch.branches matcher)))) in
  Link.output (SumFromI32.create types sum_type (SumFromI32.tag sum) values) 0

let extend types matcher links =
  match uniform matcher with
  | _ when Array.length links = 0 -> RegionMatch.selector matcher
  | Some (tag, values) -> tagged types (Array.length (RegionMatch.branches matcher)) tag (Array.append values links)
  | None -> invalid_arg "Only a uniform selector takes more values."

type 'x route = { path : ('x node * int) list; link : 'x link; outer : 'x link; within : 'x link -> 'x link }

let grouped matcher_of items =
  let matchers =
    List.fold_left
      (fun found item ->
        let matcher = matcher_of item in
        if List.memq matcher found then found else matcher :: found)
      [] items
  in
  List.rev_map (fun matcher -> (matcher, List.filter (fun item -> matcher_of item == matcher) items)) matchers

let rec route types readers routes =
  List.iter
    (fun (matcher, through) ->
      let _, values = Option.get (uniform matcher) in
      let added =
        through
        |> List.map (fun route -> route.outer)
        |> List.filter (fun link -> not (Constant.is link.node || Array.exists (Link.equal link) values))
        |> Link.distinct
      in
      let carried = Array.append values added in
      Readers.redefine_match readers matcher (extend types matcher added) (Node.output_types types matcher)
        (fun variant fields ->
          let field link = if Constant.is link.node then Constant.copy link else fields.(Link.index_of carried link) in
          let here, deeper =
            through
            |> List.filter (fun route -> snd (List.hd route.path) = variant)
            |> List.partition (fun route -> List.is_empty (List.tl route.path))
          in
          route types readers
            (List.map (fun inner -> { inner with path = List.tl inner.path; outer = field inner.outer }) deeper);
          let branch = RegionMatch.branch matcher variant in
          Rewrite.substitute branch (List.map (fun route -> (route.link, route.within (field route.outer))) here);
          Array.copy branch.results.inputs))
    (grouped (fun route -> fst (List.hd route.path)) routes)
