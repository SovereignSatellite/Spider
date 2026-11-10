let through_operation live operands results =
  List.iter (Bits.remove live) results;
  List.iter (fun operand -> ignore (Bits.add live operand)) operands

(** A try section's box for what it writes and its clauses read starts holding that variable's value, so the section
    reads it. *)
let through caught live (statement : Graph.statement) =
  match statement with
  | Operation (_, operands, results) -> through_operation live operands results
  | Exception (Try section, operands, results) ->
      let boxed = List.filter (fun output -> Array.mem output (caught ())) (Array.to_list section.outputs) in
      through_operation live (operands @ boxed) results
  | Exception (_, operands, results) -> through_operation live operands results
  | Move (target, source) ->
      Bits.remove live target;
      ignore (Bits.add live source)
  | Assign (target, _) -> Bits.remove live target

let record boundaries live id = boundaries.(id) <- Bits.to_array live

let replace_with_boundary live boundaries point =
  Bits.clear live;
  Bits.add_all live boundaries.(point)

let merge_into_boundary live boundaries head =
  Bits.add_all live boundaries.(head);
  record boundaries live head

let after graph live boundaries id =
  if Graph.is_branch_start graph id then begin
    Bits.clear live;
    List.iter (fun arm -> if arm > id then Bits.add_all live boundaries.(arm)) (Graph.block graph id).successors
  end
  else begin
    Option.iter (replace_with_boundary live boundaries) (Graph.branch_end graph id);
    Option.iter (merge_into_boundary live boundaries) (Graph.repeat_start graph id)
  end

let before graph live boundaries id =
  if Graph.is_loop_head graph id then begin
    Bits.add_all live boundaries.(id);
    record boundaries live id
  end
  else if Graph.is_branch_end graph id || Graph.is_arm_entry graph id then record boundaries live id

let block graph live boundaries id =
  after graph live boundaries id;
  Option.iter (fun selector -> ignore (Bits.add live selector)) (Graph.block graph id).selector;
  let caught () = boundaries.(List.hd (Graph.block graph id).successors) in
  List.iter (through caught live) (List.rev (Graph.block graph id).statements);
  before graph live boundaries id

let walk graph boundaries results =
  let live = Bits.of_list results in
  for id = Graph.count graph - 1 downto 0 do
    block graph live boundaries id
  done;
  live

let run graph results =
  let boundaries = Array.make (Graph.count graph) [||] in
  if List.exists (Graph.is_loop_head graph) (List.init (Graph.count graph) Fun.id) then
    ignore (walk graph boundaries results);
  record boundaries (walk graph boundaries results) 0;
  boundaries
