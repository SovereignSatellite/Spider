open IR
open Allocator

module type Target = sig
  type x
  type frame
  type expression
  type statement

  val selects_once : bool
  val layouts : frame -> x Layouts.t
  val allocation : frame -> x Allocation.t
  val reader : frame -> (x, expression) Computation.reader
  val declare : frame -> Register.t -> statement list
  val compute : frame -> x Node.t -> (expression, statement) Computation.t
  val read : frame -> Register.t -> Type.t -> expression
  val path : Register.t -> expression
  val unused : expression
  val assign : expression list * expression list -> statement
  val discard : expression -> statement
  val if_ : expression * statement list * statement list -> statement
  val break_ : statement
  val select : expression -> (int list * statement list) list -> statement list
  val loop : statement list -> statement
end

module Make (Target : Target) = struct
  open Target

  let move frame (move : x Move.t) =
    match move with
    | Copy (source, leaf_type, destination) ->
        let value = read frame source leaf_type in
        assign ([ path destination ], [ value ])
    | Swap (first, first_leaf_type, second, second_leaf_type) ->
        let second_value = read frame second second_leaf_type in
        let first_value = read frame first first_leaf_type in
        assign ([ path first; path second ], [ second_value; first_value ])
    | Compute (source, leaf, destination) ->
        let value = (reader frame).value source leaf in
        assign ([ path destination ], [ value ])

  let moves frame reader = List.map (move frame) (Allocation.moves (allocation frame) reader)

  (** One assignment reads every input before writing any register, as an output may take an input's register. *)
  let write_each frame node computed =
    let leaves =
      List.mapi
        (fun index (link, leaf) -> (computed index, Allocation.register (allocation frame) link leaf))
        (Computation.output_leaves (layouts frame) node)
    in
    let writes =
      List.filter_map
        (fun (computed, register) ->
          Option.bind computed (fun value -> Option.map (fun register -> (path register, value)) register))
        leaves
    in
    match writes with
    | [] -> List.filter_map (fun (computed, _) -> Option.map discard computed) leaves
    | _ -> [ assign (List.split writes) ]

  let write_all frame node value =
    let held =
      List.map
        (fun (link, leaf) -> Allocation.register (allocation frame) link leaf)
        (Computation.output_leaves (layouts frame) node)
    in
    let written register = match register with Some register -> path register | None -> unused in
    if List.exists Option.is_some held then [ assign (List.map written held, [ value ]) ] else [ discard value ]

  (** Alike arms keep their test where it computes the selector, which dropping the test would lose. *)
  let chosen frame matcher branches =
    let selector = RegionMatch.selector matcher in
    match (branches, Computation.runs branches) with
    | [], _ -> []
    | [ branch ], _ -> branch
    | _, [ (_, branch) ] when Option.is_none (Allocation.inlined (allocation frame) selector 0) -> branch
    | [ on_false; [] ], _ -> [ if_ ((reader frame).test selector 0, on_false, []) ]
    | [ on_false; on_true ], _ -> [ if_ ((reader frame).test selector 1, on_true, on_false) ]
    | _, runs -> select ((reader frame).value selector 0) runs

  let rec region frame region = List.concat_map (node frame) (Allocation.evaluated (allocation frame) region)

  and node frame node =
    match node.kind with
    | RegionMatch branches -> matched frame node branches
    | RegionRepeat body -> repeated frame node body
    | _ -> (
        match compute frame node with
        | Statements statements -> statements
        | Each computed -> write_each frame node computed
        | All value -> write_all frame node value)

  and matched frame matcher branches =
    chosen frame matcher
      (List.map
         (fun branch ->
           let statements = region frame branch in
           statements @ moves frame branch.results)
         (Array.to_list branches))

  and repeated frame repeat body =
    let entry = moves frame repeat in
    let following = Allocation.following (allocation frame) repeat in
    let looped = leaving frame body (RegionRepeat.step repeat) following [] @ moves frame body.results in
    entry @ [ loop looped ]

  (** The region, then [after], leaving the loop where [tested] holds variant 0: within the branches of the Match
      deciding its tag, else by a test before [after], which runs on both paths, as its moves may overwrite what the
      test reads. Where the loop leaves, it first runs the branch of the Match [following] it that the exit value
      selects. *)
  and leaving frame left tested following after =
    match Allocation.deciding (allocation frame) left tested with
    | Some (matcher, port) ->
        let preceding = List.filter (fun node -> node != matcher) (Allocation.evaluated (allocation frame) left) in
        let within (branch : x Region.t) (held : x Link.t) =
          if held.node == matcher then branch.results.inputs.(held.port) else held
        in
        List.concat_map (node frame) preceding
        @ chosen frame matcher
            (List.map
               (fun branch ->
                 leaving frame branch branch.results.inputs.(port)
                   (Option.map (fun (next, held) -> (next, within branch held)) following)
                   (moves frame branch.results @ after))
               (Array.to_list (RegionMatch.branches matcher)))
    | None -> (
        let leave () =
          match following with
          | Some (next, held) ->
              let branch = RegionMatch.branch next (Option.get (Decisions.constant held)) in
              after @ region frame branch @ moves frame branch.results @ [ break_ ]
          | None -> after @ [ break_ ]
        in
        region frame left
        @
        match Conditions.known (allocation frame) tested 0 with
        | Some true -> leave ()
        | Some false -> after
        | None -> if_ ((reader frame).test tested 0, leave (), []) :: after)

  let reads_once layouts (reader : x Node.t) input held =
    match reader.kind with
    | RegionMatch branches -> Array.length branches = 2 || (selects_once && Array.length branches > 2)
    | RegionRepeat _ -> true
    | _ -> Computation.reads_once layouts reader input held

  let body frame (definition : x Region.t) =
    let parameters =
      List.filter_map
        (fun (link, leaf) -> Allocation.register (allocation frame) link leaf)
        (Computation.output_leaves (layouts frame) definition.arguments)
    in
    let declarations =
      List.concat_map (declare frame)
        (List.filter (fun register -> not (List.mem register parameters)) (Allocation.registers (allocation frame)))
    in
    declarations @ region frame definition
end
