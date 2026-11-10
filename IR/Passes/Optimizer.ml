open IR

let changes edited table passes =
  List.fold_left
    (fun changed pass ->
      let edits = Dynarray.length edited in
      if pass !table || Dynarray.length edited > edits then begin
        table := Readers.renewed !table;
        true
      end
      else changed)
    false passes

let enabled options passes =
  List.filter_map (fun (optimization, pass) -> if options optimization then Some pass else None) passes

let simplifications options rules =
  Local.run rules
  :: enabled options
       [
         (CommonNodeElimination.optimization, CommonNodeElimination.run);
         (InvariantValueRedirection.optimization, InvariantValueRedirection.run);
         (StorageForwarding.optimization, StorageForwarding.run);
       ]

let commute types rules table = MatchCommute.run types (Local.first (rules table)) table

let restructurings options types rules =
  enabled options
    [
      (MatchCollapse.optimization, MatchCollapse.run types);
      (MatchCommute.optimization, commute types rules);
      (RepeatReduction.optimization, RepeatReduction.run);
      (Placement.optimization, Placement.run);
      (RepeatInversion.optimization, RepeatInversion.run);
      (KnownParameters.optimization, KnownParameters.run);
      (DeadPortElimination.optimization, DeadPortElimination.run);
    ]

let round simplifications restructurings region settling =
  let table = ref (Readers.create ~settling region) in
  let edited = Readers.edits settling in
  let edits = Dynarray.length edited in
  let simplified = changes edited table simplifications in
  match (changes edited table restructurings || Dynarray.length edited > edits, simplified) with
  | true, _ -> Revisit.Subtree (Dynarray.to_list edited)
  | false, true -> Revisit.Region
  | false, false -> Revisit.Nothing

let settle_bodies simplifications restructurings settled functions =
  RegionDriver.run_fixpoint
    (fun owner region ->
      let settling = Readers.settling owner in
      fun () -> round simplifications restructurings region settling)
    settled functions

let settle options extension (compilation_unit : 'x CompilationUnit.t) =
  let types = compilation_unit.types in
  let rules table = Local.rules options types table @ extension types table in
  settle_bodies
    (Local.run rules :: enabled options [ (CommonNodeElimination.optimization, CommonNodeElimination.run) ])
    (enabled options
       [
         (MatchCollapse.optimization, MatchCollapse.run types);
         (MatchCommute.optimization, commute types rules);
         (KnownParameters.optimization, KnownParameters.run);
         (DeadPortElimination.optimization, DeadPortElimination.run);
       ])
    (RegionSet.create 16)
    (Regions.functions compilation_unit.root)

let run options (compilation_unit : 'x CompilationUnit.t) =
  let root, types = (compilation_unit.root, compilation_unit.types) in
  let functions = Regions.functions root in
  let settled = RegionSet.create 64 in
  let rules = Local.rules options types in
  let stages =
    enabled options
      [
        (Inlining.optimization, fun () -> Inlining.run functions);
        (FunctionMerging.optimization, fun () -> FunctionMerging.run root functions);
      ]
  in
  let rec optimize () =
    settle_bodies (simplifications options rules) (restructurings options types rules) settled functions;
    let changed stage =
      let changed = stage () in
      if RegionSet.length changed > 0 then Some changed else None
    in
    match List.find_map changed stages with
    | Some changed ->
        RegionSet.except settled changed;
        optimize ()
    | None -> ()
  in
  optimize ()
