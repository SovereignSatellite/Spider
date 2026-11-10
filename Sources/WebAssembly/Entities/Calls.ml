open IR
open Definitions
open Entities

let apply context (state : 'x State.t) callee code environment arguments =
  let effects = Hashtbl.find context.effects callee in
  let accesses = Array.of_list (List.filter (fun field -> Fields.mem field effects) (Array.to_list state.ordered)) in
  let token =
    fence state.token (Array.map (fun field -> state.pending.(State.position state.ordered field)) accesses)
  in
  let applied = Node.create FunctionApply (Array.of_list ((code :: environment :: arguments) @ [ token ])) in
  let results = Node.output_count context.types applied - 1 in
  let returned = Link.output applied results in
  let record state field =
    let after link = fence link [| returned |] in
    match Fields.find field effects with
    | Reads -> State.after state field Reads returned
    | Writes -> State.after state field Writes (after (State.contents context state field Reads))
    | Replaces ->
        let box = after (State.item state field) in
        State.replaced state field box (held context field box)
  in
  (List.init results (Link.output applied), Array.fold_left record { state with token = returned } accesses)

let parts context state index =
  let held = State.item state (Function index) in
  if index < context.imported_functions then
    let opened =
      RegionMatch.create context.types held [ Context.function_of context index; context.erased ] (fun _ parts -> parts)
    in
    (Link.output opened 0, Link.output opened 1)
  else (make (RegionReference context.bodies.(index - context.imported_functions)) [||], held)

let function_value context state index =
  if index < context.imported_functions then State.item state (Function index)
  else
    let code, environment = parts context state index in
    make
      (SumNew { sum_type = Context.closure context (Link.type_ context.types code); variant = 0 })
      [| code; environment |]

let call context state index arguments =
  let code, environment = parts context state index in
  apply context state (Function index) code environment arguments

let reference context state index =
  let code, environment = parts context state index in
  let erased = make (FunctionRetype (Context.erased_function context.types)) [| code |] in
  make (SumNew { sum_type = context.signatures.reference; variant = 1 }) [| erased; environment |]
