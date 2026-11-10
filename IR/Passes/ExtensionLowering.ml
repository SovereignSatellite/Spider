open IR
open Definitions

let bounded types options omission node outside access out_of_bounds =
  if options omission then access node.inputs
  else
    let result_type = Node.output_type types node 0 in
    Branch.choose_one types (outside node.inputs) node.inputs result_type access (fun _ -> out_of_bounds result_type)

let allocation types allocation create allocated filled =
  let result_type = Node.output_type types allocation 0 in
  let length, initial = (allocation.inputs.(0), allocation.inputs.(1)) in
  let array = create length initial in
  Branch.choose_one types (allocated array) [| array; length; initial |] result_type
    (fun _ -> Link.output (Node.create (SumNew { sum_type = result_type; variant = 0 }) [||]) 0)
    (fun fields -> Link.output (Node.create (SumNew { sum_type = result_type; variant = 1 }) [| filled fields |]) 0)

let attempt types attempt call threw returned caught =
  let outcome_type = Node.output_type types attempt 0 in
  let given = call (Node.inputs attempt) in
  Branch.choose_one types
    (threw given.(0))
    given outcome_type
    (fun fields -> Link.output (Node.create (SumNew { sum_type = outcome_type; variant = 1 }) (returned fields)) 0)
    (fun fields -> Link.output (Node.create (SumNew { sum_type = outcome_type; variant = 0 }) (caught fields)) 0)

let run lower (compilation_unit : 'x CompilationUnit.t) =
  let rule node = Option.map (fun replacement -> [| replacement |]) (lower node) in
  ignore
    (Bodies.in_parallel
       (fun added body -> ignore (Rewrite.nested rule added body))
       (Regions.functions compilation_unit.root))
