type handling = Try of outlined | Catch of int

and statement =
  | Operation of Wasm.Ast.instr' * int list * int list
  | Move of int * int
  | Assign of int * int
  | Exception of handling * int list * int list

and basic_block = {
  statements : statement list;
  selector : int option;
  mutable predecessors : int list;
  mutable successors : int list;
}

and outlined = { graph : basic_block Dynarray.t; outputs : int array; boundaries : int array array }

type t = basic_block Dynarray.t

let block graph id = Dynarray.get graph id
let count graph = Dynarray.length graph

let add graph statements selector =
  Dynarray.add_last graph { statements; selector; predecessors = []; successors = [] };
  Dynarray.length graph - 1

let no_operation graph = add graph [] None
let assignment graph variable value = add graph [ Assign (variable, value) ] None
let selection graph variable = add graph [] (Some variable)

let assigns graph variable id =
  match (block graph id).statements with [ Assign (target, _) ] -> target = variable | _ -> false

let add_edge graph from target =
  (block graph from).successors <- (block graph from).successors @ [ target ];
  (block graph target).predecessors <- (block graph target).predecessors @ [ from ]

let index_of item items = Option.get (List.find_index (Int.equal item) items)

let replace_edge graph from target replacement =
  let successors = (block graph from).successors in
  let position = index_of target successors in
  (block graph from).successors <- List.mapi (fun index id -> if index = position then replacement else id) successors;
  (block graph replacement).predecessors <- (block graph replacement).predecessors @ [ from ];
  let predecessors = (block graph target).predecessors in
  let position = index_of from predecessors in
  (block graph target).predecessors <- List.filteri (fun index _ -> index <> position) predecessors

let add_predecessors graph =
  for from = count graph - 1 downto 0 do
    List.iter
      (fun target -> (block graph target).predecessors <- from :: (block graph target).predecessors)
      (List.rev (block graph from).successors)
  done

(** Visiting successors last first numbers them in their listed order where neither reaches the other. *)
let rec reverse_postorder graph visited order id =
  if visited.(id) then order
  else begin
    visited.(id) <- true;
    id :: List.fold_left (reverse_postorder graph visited) order (List.rev (block graph id).successors)
  end

let compact graph =
  let order = reverse_postorder graph (Array.make (count graph) false) [] 0 in
  let numbers = Array.make (count graph) (-1) in
  List.iteri (fun number id -> numbers.(id) <- number) order;
  let compacted = Dynarray.create () in
  List.iter
    (fun id ->
      let original = block graph id in
      Dynarray.add_last compacted
        {
          original with
          predecessors =
            List.filter_map
              (fun predecessor -> if numbers.(predecessor) >= 0 then Some numbers.(predecessor) else None)
              original.predecessors;
          successors = List.map (Array.get numbers) original.successors;
        })
    order;
  compacted

let is_branch_start graph id =
  List.length (List.filter (fun successor -> successor > id) (block graph id).successors) >= 2

let is_branch_end graph id =
  List.length (List.filter (fun predecessor -> predecessor < id) (block graph id).predecessors) >= 2

let is_arm_entry graph id =
  List.exists (fun predecessor -> predecessor < id && is_branch_start graph predecessor) (block graph id).predecessors

let is_loop_head graph id = List.exists (fun predecessor -> predecessor >= id) (block graph id).predecessors

let branch_end graph id =
  List.find_opt (fun successor -> successor > id && is_branch_end graph successor) (block graph id).successors

let repeat_start graph id = List.find_opt (fun successor -> successor <= id) (block graph id).successors
