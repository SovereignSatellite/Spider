open IR
open Types

type label = { target : int; base : int; carried : Type.t list }
type construct = { label : label; below : Type.t list; signature : signature; continuation : int; unreachable : bool }

type state = {
  variables : Variables.t;
  blocks : Graph.t;
  current : int;
  statements : Graph.statement list;
  stack : Type.t list;
  constructs : construct list;
}

let onto stack items = List.rev_append items stack
let allocate state = Graph.no_operation state.blocks

let finish_block state id statements selector successors =
  Dynarray.set state.blocks id { statements; selector; predecessors = []; successors }

let finish state selector successors = finish_block state state.current (List.rev state.statements) selector successors
let start state id stack = { state with current = id; statements = []; stack }
let innermost state = List.hd state.constructs
let outermost state = List.nth state.constructs (List.length state.constructs - 1)

(** Where code cannot fall through, WebAssembly lets the stack supply whatever a pop needs, so pops stop at [below]. *)
let pop state types =
  let available = List.length state.stack - List.length (innermost state).below in
  let stack = List.drop (min (List.length types) available) state.stack in
  let height = List.length stack in
  ({ state with stack }, List.mapi (fun index popped -> Variables.slot state.variables (height + index) popped) types)

let push state types =
  let height = List.length state.stack in
  ( { state with stack = onto state.stack types },
    List.mapi (fun index pushed -> Variables.slot state.variables (height + index) pushed) types )

let moves state target types =
  let source = List.length state.stack - List.length types in
  if (innermost state).unreachable || source = target then []
  else
    List.mapi
      (fun index moved ->
        Graph.Move
          (Variables.slot state.variables (target + index) moved, Variables.slot state.variables (source + index) moved))
      types

let jump_through_edge state label moved =
  let edge = allocate state in
  finish_block state edge moved None [ label.target ];
  edge

let jump state label =
  match moves state label.base label.carried with [] -> label.target | moved -> jump_through_edge state label moved

let leave state selector successors =
  finish state selector successors;
  let next = allocate state in
  let construct = innermost state in
  start
    { state with constructs = { construct with unreachable = true } :: List.tl state.constructs }
    next construct.below

let emit state statement = { state with statements = statement :: state.statements }
let label state depth = (List.nth state.constructs (index depth)).label

let compare state comparison operand constant =
  let state = emit state (Assign (Variable.d, constant)) in
  emit state (Operation (comparison, [ operand; Variable.d ], [ Variable.d ]))

let diamond state on_false on_true =
  let false_block = allocate state in
  let true_block = allocate state in
  let next = allocate state in
  finish state (Some Variable.d) [ false_block; true_block ];
  finish_block state false_block on_false None [ next ];
  finish_block state true_block on_true None [ next ];
  start state next state.stack

let is_true state value = compare state (Compare (I32 Ne)) value 0

let enter state continuation target signature carried =
  let below, _ = pop state signature.parameters in
  let construct =
    {
      label = { target; base = List.length below.stack; carried };
      below = below.stack;
      signature;
      continuation;
      unreachable = false;
    }
  in
  { state with stack = onto below.stack signature.parameters; constructs = construct :: state.constructs }

let open_block signatures state block_type =
  let signature = Stack.of_block signatures block_type in
  let continuation = allocate state in
  enter state continuation continuation signature signature.results

let open_loop signatures state block_type =
  let signature = Stack.of_block signatures block_type in
  let head = allocate state in
  let continuation = allocate state in
  finish state None [ head ];
  enter (start state head state.stack) continuation head signature signature.parameters

(** An [if] without an [else] must give back its parameters' types, so its false edge needs no moves. *)
let open_if signatures state block_type =
  let signature = Stack.of_block signatures block_type in
  let state, selector = pop state [ Type.i32 ] in
  let state = is_true state (List.hd selector) in
  let then_block = allocate state in
  let continuation = allocate state in
  finish state (Some Variable.d) [ continuation; then_block ];
  enter (start state then_block state.stack) continuation continuation signature signature.results

let open_else state head =
  let construct = innermost state in
  finish state None [ jump state construct.label ];
  let else_block = allocate state in
  let head = Graph.block state.blocks head in
  head.successors <- else_block :: List.tl head.successors;
  let state = { state with constructs = { construct with unreachable = false } :: List.tl state.constructs } in
  start state else_block (onto construct.below construct.signature.parameters)

let close state =
  let construct = innermost state in
  let results = construct.signature.results in
  finish
    { state with statements = onto state.statements (moves state construct.label.base results) }
    None [ construct.continuation ];
  start { state with constructs = List.tl state.constructs } construct.continuation (onto construct.below results)

let table state labels default =
  let state, selector = pop state [ Type.i32 ] in
  let selector = List.hd selector in
  let labels = List.map (label state) labels in
  let state =
    if labels = [] then state
    else
      let last = List.length labels in
      let state = compare state (Compare (I32 (Lt U))) selector last in
      diamond state [ Assign (selector, last) ] []
  in
  let successors = List.map (jump state) (labels @ [ label state default ]) in
  leave state (if List.length successors > 1 then Some selector else None) successors

let branch state depth stack =
  let taken = jump state (label state depth) in
  let next = allocate state in
  finish state (Some Variable.d) [ next; taken ];
  start state next stack

let conditional state depth =
  let state, selector = pop state [ Type.i32 ] in
  branch (is_true state (List.hd selector)) depth state.stack

let on_null state depth =
  let reference = List.hd state.stack in
  let state, read = pop state [ reference ] in
  branch (emit state (Operation (RefIsNull, read, [ Variable.d ]))) depth (onto state.stack [ reference ])

(** The label takes the reference, so it leaves before the reference is popped, where D is 0. *)
let on_non_null state depth =
  let popped, read = pop state [ List.hd state.stack ] in
  let state = emit state (Operation (RefIsNull, read, [ Variable.d ])) in
  let taken = jump state (label state depth) in
  let next = allocate state in
  finish state (Some Variable.d) [ taken; next ];
  start state next popped.stack

let select state =
  let value_type = List.nth state.stack 1 in
  let state, operands = pop state [ value_type; value_type; Type.i32 ] in
  let state, _ = push state [ value_type ] in
  let operands = Array.of_list operands in
  diamond (is_true state operands.(2)) [ Move (operands.(0), operands.(1)) ] []

let returning state = leave state None [ jump state (outermost state).label ]

let on_reference state instruction result =
  let reference = List.hd state.stack in
  let state, read = pop state [ reference ] in
  let state, written = push state [ result reference ] in
  emit state (Operation (instruction, read, written))

let operation signatures state instruction =
  let signature = Stack.change signatures instruction in
  let state, read = pop state signature.parameters in
  let state, written = push state signature.results in
  emit state (Operation (instruction, read, written))

let unreachable signatures state instruction =
  leave (operation signatures state instruction) None [ (outermost state).continuation ]

let local state local =
  let variable = Variables.local (index local) in
  (variable, Variables.type_ state.variables variable)

let drop_value state = fst (pop state [ List.hd state.stack ])

let get_local state index =
  let variable, local_type = local state index in
  let state, written = push state [ local_type ] in
  emit state (Move (List.hd written, variable))

let set_local state index =
  let variable, local_type = local state index in
  let state, read = pop state [ local_type ] in
  emit state (Move (variable, List.hd read))

let tee_local state index =
  let variable, local_type = local state index in
  let state, read = pop state [ local_type ] in
  let state, _ = push state [ local_type ] in
  emit state (Move (variable, List.hd read))

let unconditional state depth = leave state None [ jump state (label state depth) ]

let step signatures state (instruction : Wasm.Ast.instr') =
  match instruction with
  | _ when (innermost state).unreachable -> state
  | Nop -> state
  | Drop -> drop_value state
  | Select _ -> select state
  | LocalGet index -> get_local state index
  | LocalSet index -> set_local state index
  | LocalTee index -> tee_local state index
  | Br depth -> unconditional state depth
  | BrIf depth -> conditional state depth
  | BrOnNull depth -> on_null state depth
  | BrOnNonNull depth -> on_non_null state depth
  | BrTable (labels, default) -> table state labels default
  | Return -> returning state
  | Unreachable | Throw _ | ThrowRef -> unreachable signatures state instruction
  | RefIsNull -> on_reference state instruction (Fun.const Type.i32)
  | RefAsNonNull -> on_reference state instruction Fun.id
  | _ -> operation signatures state instruction

let written graph =
  let variables = Bits.create () in
  Dynarray.iter
    (fun (block : Graph.basic_block) ->
      List.iter
        (fun (statement : Graph.statement) ->
          match statement with
          | Operation (_, _, results) | Exception (_, _, results) ->
              List.iter (fun result -> ignore (Bits.add variables result)) results
          | Move (target, _) | Assign (target, _) -> ignore (Bits.add variables target))
        block.statements)
    graph;
  Array.of_list
    (List.filter (fun variable -> variable >= Variable.first_local) (Array.to_list (Bits.to_array variables)))

let caught_values signatures state tag =
  let state, values = push state signatures.tags.(index tag) in
  emit state (Exception (Catch (index tag), [ Variable.x ], values @ [ Variable.d ]))

let caught_reference signatures state =
  let state, reference = push state [ signatures.exception_ ] in
  emit state (Move (List.hd reference, Variable.x))

let catch_all state depth = finish state None [ jump state (label state depth) ]

let rec clauses signatures state raising (catches : Wasm.Ast.catch list) =
  match catches with
  | [] -> finish state None [ raising ]
  | caught :: rest -> (
      let catch matched depth = clauses signatures (branch matched depth state.stack) raising rest in
      match caught.it with
      | Catch (tag, depth) -> catch (caught_values signatures state tag) depth
      | CatchRef (tag, depth) -> catch (caught_reference signatures (caught_values signatures state tag)) depth
      | CatchAll depth -> catch_all state depth
      | CatchAllRef depth -> catch_all (caught_reference signatures state) depth)

(** A trap's null goes on unwinding before any clause sees it. *)
let dispatch signatures state catches =
  let raising = allocate state in
  let tagged = allocate state in
  let state = emit state (Operation (RefIsNull, [ Variable.x ], [ Variable.d ])) in
  finish state (Some Variable.d) [ tagged; raising ];
  let exit = (outermost state).continuation in
  finish_block state raising [ Operation (ThrowRef, [ Variable.x ], []) ] None [ exit ];
  clauses signatures (start state tagged state.stack) raising catches

let rec instructions signatures state code = List.fold_left (instruction signatures) state code

and instruction signatures state (instruction : Wasm.Ast.instr) =
  match instruction.it with
  | Block (block_type, body) -> close (instructions signatures (open_block signatures state block_type) body)
  | Loop (block_type, body) -> close (instructions signatures (open_loop signatures state block_type) body)
  | If (block_type, taken, []) -> close (instructions signatures (open_if signatures state block_type) taken)
  | If (block_type, taken, otherwise) ->
      let finished = instructions signatures (open_if signatures state block_type) taken in
      close (instructions signatures (open_else finished state.current) otherwise)
  | TryTable (block_type, catches, body) ->
      if (innermost state).unreachable then state else try_table signatures state block_type catches body
  | other -> step signatures state other

and section signatures outer code signature : Graph.outlined =
  let blocks = Dynarray.create () in
  let entry = Graph.no_operation blocks in
  let exit = Graph.no_operation blocks in
  let leaving way =
    let stub = Graph.assignment blocks Variable.e way in
    (Graph.block blocks stub).successors <- [ exit ];
    stub
  in
  let finished = leaving 1 in
  let exits =
    List.mapi
      (fun index construct ->
        { construct with label = { construct.label with target = leaving (index + 2) }; continuation = finished })
      outer.constructs
  in
  let state = { outer with blocks; current = entry; statements = []; constructs = exits } in
  ignore (close (instructions signatures (enter state finished finished signature signature.results) code));
  Graph.add_predecessors blocks;
  let graph = Restructuring.run blocks in
  let outputs = written graph in
  let boundaries = Liveness.run graph (Variable.e :: Array.to_list outputs) in
  { graph; outputs; boundaries }

and try_table signatures state block_type catches body =
  let signature = Stack.of_block signatures block_type in
  let built = section signatures state body signature in
  let below, _ = pop state signature.parameters in
  let state =
    emit below
      (Exception
         (Try built, Array.to_list built.boundaries.(0), Array.to_list built.outputs @ [ Variable.e; Variable.x ]))
  in
  let caught = allocate state in
  let continuation = allocate state in
  let codes =
    Dynarray.fold_left
      (fun codes (block : Graph.basic_block) ->
        List.fold_left
          (fun codes (statement : Graph.statement) ->
            match statement with Assign (variable, code) when variable = Variable.e -> max codes code | _ -> codes)
          codes block.statements)
      1 built.graph
  in
  let targets = List.map (fun construct -> construct.label.target) (List.take (codes - 1) state.constructs) in
  finish state (Some Variable.e) (caught :: continuation :: targets);
  dispatch signatures (start state caught below.stack) catches;
  start state continuation (onto below.stack signature.results)

let variables signatures signature (code : Wasm.Ast.func) =
  let (Func (_, locals, _)) = code.it in
  Variables.create
    (Array.of_list
       (signature.parameters
       @ List.map
           (fun (local : Wasm.Ast.local) ->
             let (Local local_type) = local.it in
             value_type signatures local_type)
           locals))
    signature.results
    [ signatures.reference; signatures.exception_; signatures.host ]
    signatures.exception_

let build signatures (variables : Variables.t) (code : Wasm.Ast.func) =
  let (Func (_, _, body)) = code.it in
  let results = variables.result_types in
  let blocks = Dynarray.create () in
  let entry = Graph.no_operation blocks in
  let exit = Graph.no_operation blocks in
  let state =
    {
      variables;
      blocks;
      current = entry;
      statements = [];
      stack = [];
      constructs =
        [
          {
            label = { target = exit; base = 0; carried = results };
            below = [];
            signature = { parameters = []; results };
            continuation = exit;
            unreachable = false;
          };
        ];
    }
  in
  finish (close (instructions signatures state body)) None [];
  Graph.add_predecessors blocks;
  blocks
