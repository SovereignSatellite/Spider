open Definitions

let create types selector output_types build_branch =
  let branch variant fields =
    Region.create ~build:(fun branch -> build_branch variant (Region.parameters branch)) types fields output_types
  in
  Node.create
    (RegionMatch (Array.of_list (List.mapi branch (TypeArena.variants types (Link.type_ types selector)))))
    [| selector |]

let branches matcher =
  match matcher.kind with RegionMatch branches -> branches | _ -> invalid_arg "Expected a Region.Match."

let selector matcher = matcher.inputs.(0)
let branch matcher variant = (branches matcher).(variant)
let field matcher variant index = Region.parameter (branch matcher variant) index
let fields matcher variant = Region.parameters (branch matcher variant)

let redefine types matcher selector output_types build_branch =
  let variants = TypeArena.variants types (Link.type_ types selector) in
  matcher.inputs.(0) <- selector;
  List.iteri
    (fun variant fields ->
      Region.redefine (branch matcher variant) fields output_types (fun branch ->
          build_branch variant (Region.parameters branch)))
    variants
