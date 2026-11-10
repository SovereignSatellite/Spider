open Definitions

let step_type types initial_types exit_types = TypeArena.intern types (Sum [ exit_types; initial_types ])

let create types initial exit_types build =
  let initial_types = Link.types types initial in
  let body =
    Region.create
      ~build:(fun body -> [| build (Region.parameters body) |])
      types initial_types
      [ step_type types initial_types exit_types ]
  in
  Node.create (RegionRepeat body) initial

let body repeat = match repeat.kind with RegionRepeat body -> body | _ -> invalid_arg "Expected a Region.Repeat."
let step repeat = (body repeat).results.inputs.(0)

let redefine types repeat initial exit_types build =
  let initial_types = Link.types types initial in
  repeat.inputs <- initial;
  Region.redefine (body repeat) initial_types
    [ step_type types initial_types exit_types ]
    (fun body -> [| build (Region.parameters body) |])
