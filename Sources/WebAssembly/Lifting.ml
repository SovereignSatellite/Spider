open IR
open Definitions
open Entities
open Traps

type 'x scope = {
  context : 'x context;
  graph : Graph.t;
  variables : Variables.t;
  boundaries : int array array;
  (* One array serves every path, since the entry and each region's start and end set the variables live there. *)
  held : 'x link option array;
  (* The section's writes the clauses read, and in their order each one's box and the value the box last took. *)
  boxed : int array;
  boxes : 'x link array;
  stored : 'x link option array;
}

let read scope variable = Option.get scope.held.(variable)
let hold scope variable value = scope.held.(variable) <- Some value

let flush scope (state : 'x State.t) =
  let written = Dynarray.create () in
  Array.iteri
    (fun index variable ->
      if not (Option.equal Link.equal scope.held.(variable) scope.stored.(index)) then begin
        scope.stored.(index) <- scope.held.(variable);
        scope.boxes.(index) <-
          result (Write.box_set scope.context.types (fence scope.boxes.(index) [| state.token |]) (read scope variable));
        Dynarray.add_last written scope.boxes.(index)
      end)
    scope.boxed;
  if Dynarray.length written = 0 then state else { state with token = fence state.token (Dynarray.to_array written) }

let carried scope state variables = Array.concat [ State.links state; scope.boxes; Array.map (read scope) variables ]

(** Every region starts and ends with each box holding its variable's value. *)
let received scope state variables links =
  let start = Array.length links - Array.length variables in
  Array.iteri (fun index variable -> hold scope variable links.(start + index)) variables;
  Array.blit links (start - Array.length scope.boxes) scope.boxes 0 (Array.length scope.boxes);
  Array.iteri (fun index variable -> scope.stored.(index) <- scope.held.(variable)) scope.boxed;
  State.of_links state links

let operate scope state instruction operands results =
  let state = if Operations.throws instruction then flush scope state else state in
  let outputs, state = Operations.lift scope.context state instruction (List.map (read scope) operands) in
  List.iter2 (hold scope) results outputs;
  state

(** Each variable the section writes that the clauses read goes in a box, since where the section throws those boxes are
    all that remains of what it wrote. *)
let section_outputs scope id (section : Graph.outlined) =
  let successors = (Graph.block scope.graph id).successors in
  let caught = scope.boundaries.(List.hd successors) in
  let boxed, unboxed = List.partition (fun variable -> Array.mem variable caught) (Array.to_list section.outputs) in
  let continuing = Bits.create () in
  List.iter (fun successor -> Bits.add_all continuing scope.boundaries.(successor)) (List.tl successors);
  (Array.of_list boxed, Array.of_list (List.filter (Bits.contains continuing) unboxed @ [ Variable.e ]))

let returned context outcome returned_types =
  let opened = traps_in context outcome 0 returned_types Fun.id in
  Array.append (Array.init (List.length returned_types) (Link.output opened)) [| zero context.signatures.exception_ |]

let unwound context outcome entered returning variables =
  let caught = traps_in context outcome 1 (payload_types context) Fun.id in
  Array.concat
    [
      State.links (State.unwound context entered (Link.output caught 1));
      Array.map (fun variable -> zero (Variables.type_ variables variable)) returning;
      [| Link.output caught 0 |];
    ]

let opened_outcome scope state outcome links returned_types returning =
  let context = scope.context in
  let result_types = returned_types @ [ context.signatures.exception_ ] in
  let opened =
    RegionMatch.create context.types
      (select context.types 2 (make I32FromSum [| outcome |]) (Array.append [| outcome |] links))
      result_types
      (fun variant fields ->
        if variant = 1 then returned context fields.(0) returned_types
        else
          unwound context fields.(0)
            (State.of_links state (Array.sub fields 1 (Array.length fields - 1)))
            returning scope.variables)
  in
  Array.init (List.length result_types) (Link.output opened)

let resumed scope state links returning boxed boxes outputs =
  let state = State.of_links state (Array.sub outputs 0 links) in
  let reread box = result (BoxGet.create scope.context.types (fence box [| state.token |])) in
  Array.iter2 (hold scope)
    (Array.append returning [| Variable.x |])
    (Array.sub outputs links (Array.length outputs - links));
  Array.iter2 (fun variable box -> hold scope variable (reread box)) boxed boxes;
  state

let rec statement scope id state (statement : Graph.statement) =
  match statement with
  | Move (target, source) ->
      scope.held.(target) <- scope.held.(source);
      state
  | Assign (target, value) ->
      hold scope target (i32 value);
      state
  | Operation (instruction, operands, results) -> operate scope state instruction operands results
  | Exception (Try section, operands, _) -> attempt scope state id section operands
  | Exception (Catch tag, operands, results) ->
      List.iter2 (hold scope) results (Exceptions.catch scope.context state tag (read scope (List.hd operands)));
      state

and sequence scope state id stop =
  if id = stop then state
  else if Graph.is_loop_head scope.graph id then
    let state, exit = loop scope state id in
    sequence scope state exit stop
  else blocks scope state id stop

and blocks scope state id stop =
  let state = List.fold_left (statement scope id) state (Graph.block scope.graph id).statements in
  if Graph.is_branch_start scope.graph id then
    let state, merge = branch scope state id in
    sequence scope state merge stop
  else sequence scope state (List.find (fun successor -> successor > id) (Graph.block scope.graph id).successors) stop

(** The latch's selector is the step of a [RegionRepeat], which exits on variant 0, the latch's first successor. *)
and loop scope state head =
  let types = scope.context.types in
  let latch = List.find (fun predecessor -> predecessor >= head) (Graph.block scope.graph head).predecessors in
  let exit = List.hd (Graph.block scope.graph latch).successors in
  let variables = scope.boundaries.(head) in
  let state = flush scope state in
  let initial = carried scope state variables in
  let repeat =
    RegionRepeat.create types initial (Link.types types initial) (fun inside ->
        let body = flush scope (blocks scope (received scope state variables inside) head exit) in
        select types 2 (read scope (Option.get (Graph.block scope.graph latch).selector)) (carried scope body variables))
  in
  (received scope state variables (Array.init (Array.length initial) (Link.output repeat)), exit)

and branch scope state head =
  let types = scope.context.types in
  let arms = Array.of_list (Graph.block scope.graph head).successors in
  let merge = Option.get (Graph.branch_end scope.graph (arms.(1) - 1)) in
  let entered = Bits.create () in
  Array.iter (fun arm -> Bits.add_all entered scope.boundaries.(arm)) arms;
  let variables = Bits.to_array entered in
  let merged = scope.boundaries.(merge) in
  let state = flush scope state in
  let merged_types =
    Link.types types (carried scope state [||]) @ List.map (Variables.type_ scope.variables) (Array.to_list merged)
  in
  let node =
    RegionMatch.create types
      (select types (Array.length arms)
         (read scope (Option.get (Graph.block scope.graph head).selector))
         (carried scope state variables))
      merged_types
      (fun variant fields ->
        let entry = received scope state variables fields in
        carried scope (flush scope (sequence scope entry arms.(variant) merge)) merged)
  in
  (received scope state merged (Array.init (List.length merged_types) (Link.output node)), merge)

and attempt scope state id section inputs =
  let context = scope.context in
  let types = context.types in
  let boxed, returning = section_outputs scope id section in
  (* The builder counts all the section's writes as live at its end, as only here is known what follows it. *)
  let section =
    { section with boundaries = Liveness.run section.graph (Array.to_list (Array.append returning boxed)) }
  in
  let boxes =
    Array.map
      (fun variable ->
        make (BoxNew (context.boxes (Variables.type_ scope.variables variable))) [| read scope variable |])
      boxed
  in
  let passed =
    List.filter (fun variable -> Array.mem variable section.boundaries.(0) && not (Array.mem variable boxed)) inputs
  in
  let links = State.links state in
  let arguments = Array.concat [ links; Array.of_list (List.map (read scope) passed); boxes ] in
  let returned_types = Link.types types links @ List.map (Variables.type_ scope.variables) (Array.to_list returning) in
  let body =
    Region.create types (Link.types types arguments) returned_types ~build:(fun outlined ->
        lift_section scope section boxed returning state passed (Region.parameters outlined))
  in
  let outcome =
    make (FunctionTry (payload_types context)) (Array.append [| make (RegionReference body) [||] |] arguments)
  in
  resumed scope state (Array.length links) returning boxed boxes
    (opened_outcome scope state outcome links returned_types returning)

and lift_section scope section boxed returning state passed parameters =
  let links = Array.length (State.links state) in
  let first_box = links + List.length passed in
  let inside =
    {
      scope with
      graph = section.graph;
      boundaries = section.boundaries;
      held = Array.make scope.variables.count None;
      boxed;
      boxes = Array.sub parameters first_box (Array.length parameters - first_box);
      stored = Array.make (Array.length boxed) None;
    }
  in
  let entered = State.of_links state (Array.sub parameters 0 links) in
  List.iteri (fun position variable -> hold inside variable parameters.(links + position)) passed;
  Array.iteri
    (fun index variable ->
      if Array.mem variable section.boundaries.(0) then begin
        let contents = result (BoxGet.create scope.context.types inside.boxes.(index)) in
        inside.boxes.(index) <- fence inside.boxes.(index) [| contents |];
        hold inside variable contents;
        inside.stored.(index) <- Some contents
      end)
    boxed;
  let exit = flush inside (sequence inside entered 0 (Graph.count section.graph - 1)) in
  Array.append (State.links exit) (Array.map (read inside) returning)

let lift context index fields environment code arguments =
  let signature = context.signatures.functions.(index) in
  let variables = Builder.variables context.signatures signature code in
  let graph = Restructuring.run (Builder.build context.signatures variables code) in
  let boundaries = Liveness.run graph variables.results in
  let scope =
    {
      context;
      graph;
      variables;
      boundaries;
      held = Array.make variables.count None;
      boxed = [||];
      boxes = [||];
      stored = [||];
    }
  in
  let parameters = List.length signature.parameters in
  let entry =
    State.enter context index fields environment (Region.parameter arguments 0)
      (Region.parameter arguments (parameters + 1))
  in
  let initial variable =
    let local = variable - Variables.local 0 in
    if local >= 0 && local < parameters then Region.parameter arguments (local + 1)
    else zero (Variables.type_ variables variable)
  in
  Array.iter (fun variable -> hold scope variable (initial variable)) boundaries.(0);
  (* The last block, the restructuring's exit, has no statements, so stopping before it skips nothing. *)
  let exit = sequence scope entry 0 (Graph.count graph - 1) in
  Array.of_list (List.map (read scope) variables.results @ [ (State.settle exit).token ])
