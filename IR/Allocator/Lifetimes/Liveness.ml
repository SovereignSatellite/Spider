open IR

type 'x t = {
  ports : 'x Ports.t;
  flows : 'x Flows.t;
  inlined : bool array;
  apart : bool array;
  last_uses : int option array;
  definitions : int array;
  following : ('x Node.t * 'x Link.t) option array;
}

(** For each value that is a loop's argument, whether a node of the loop's body reads it. *)
let argument_reads schedule ports =
  let reads = Array.make (Ports.count ports) None in
  for number = 0 to Schedule.count schedule - 1 do
    match (Schedule.node schedule number).kind with
    | RegionRepeat body ->
        let arguments = Schedule.start schedule body in
        Array.fill reads (Ports.node_start ports arguments)
          (Ports.node_end ports arguments - Ports.node_start ports arguments)
          (Some false);
        for reader = arguments + 1 to Schedule.end_ schedule body - 1 do
          Array.iter
            (fun (input : 'x Link.t) ->
              if input.node == body.arguments then
                let port = Ports.port_of_link ports input in
                Array.fill reads (Ports.start ports port) (Ports.end_ ports port - Ports.start ports port) (Some true))
            (Schedule.node schedule reader).inputs
        done
    | _ -> ()
  done;
  reads

let create schedule ports flows reads_once computed_where_read =
  let count = Schedule.count schedule in
  let definitions = Array.make count 0 in
  let evaluations = Array.make count 0 in
  let consumers = Array.make count Flows.Nothing in
  let inlined = Array.make count false in
  let apart = Array.make count false in
  let last_uses = Array.make (Ports.count ports) None in
  let readers = Array.make (Ports.count ports) Flows.Nothing in
  let passed_on = Array.make (Ports.count ports) None in
  let next = ref (2 * count) in
  let take () =
    decr next;
    !next
  in
  let replaced = Array.make (Ports.count ports) false in
  let tested_at_end = Array.make (Ports.count ports) false in
  let followed = Array.make count false in
  let following = Array.make count None in
  Decisions.decided schedule flows
    (fun test -> replaced.(Ports.value ports test 0) <- true)
    (fun result -> tested_at_end.(Ports.value ports result 0) <- true)
    (fun repeat ((matcher, _) as next) ->
      following.(Schedule.number schedule repeat) <- Some next;
      followed.(Schedule.number schedule matcher) <- true);
  let argument_reads = argument_reads schedule ports in
  let use_at position value =
    if not (Flows.undefined flows value) then
      match last_uses.(value) with
      | Some last when last >= position -> ()
      | Some _ | None -> last_uses.(value) <- Some position
  in
  let read_at position number input value =
    if not replaced.(value) then begin
      use_at position value;
      let congruence = Flows.congruence flows value in
      readers.(congruence) <-
        (match readers.(congruence) with Nothing -> Sole (number, input) | Sole _ | Several -> Several)
    end
  in
  (* A loop's argument is read later in the walk, where its body is, so a copy into one is kept unless nothing reads it;
     any other value is read earlier. *)
  let pass_on position number (edge : Flows.edge) =
    passed_on.(edge.source) <- Some number;
    let needed =
      Option.value argument_reads.(edge.destination) ~default:(Option.is_some last_uses.(edge.destination))
    in
    if edge.copied && (needed || tested_at_end.(edge.source)) then begin
      use_at position edge.source;
      readers.(Flows.congruence flows edge.source) <- Several
    end
    else if needed then use_at position edge.source
  in
  (* A copy that is a reader's only edge runs wherever the reader does, so it reads its source as any input is read
     where its destination is live, since only then is it evaluated, unless it goes into its own region's arguments,
     which the loop does only when it repeats. *)
  let copies_once start number edges (edge : Flows.edge) =
    let destination = Ports.node_of ports (Ports.port_of ports edge.destination) in
    edge.copied
    && Array.length edges = 1
    && Option.is_some last_uses.(edge.destination)
    && not (start <= destination && destination < number)
  in
  let evaluate start position number =
    evaluations.(number) <- position;
    let edges = Flows.edges flows number in
    Array.iter (fun edge -> if not (copies_once start number edges edge) then pass_on position number edge) edges;
    for input = 0 to Ports.input_count ports number - 1 do
      let port = Ports.input_port ports number input in
      let producer = Ports.node_of ports port in
      consumers.(producer) <- (match consumers.(producer) with Nothing -> Sole number | Sole _ | Several -> Several);
      for value = Ports.start ports port to Ports.end_ ports port - 1 do
        if not (Option.equal Int.equal passed_on.(value) (Some number)) then read_at position number input value
      done
    done
  in
  let rec sole_dependent reader number =
    match consumers.(number) with
    | Sole consumer -> consumer = reader || sole_dependent reader consumer
    | Nothing | Several -> false
  in
  let held number =
    let total = ref 0 in
    for value = Ports.node_start ports number to Ports.node_end ports number - 1 do
      if Option.is_some last_uses.(value) then incr total
    done;
    !total
  in
  (* The function's results read only what is done before it returns. *)
  let returns = Schedule.end_ schedule (Schedule.body schedule) - 1 in
  (* A reader computing nothing, as a fence reads what it waits for, would never evaluate the value. *)
  let reader_within start finish number value =
    match readers.(Flows.congruence flows value) with
    | Sole (reader, input)
      when start <= reader && reader < finish && reader <> returns
           && (not (Flows.computes_nothing flows reader))
           && sole_dependent reader number
           && reads_once (Schedule.node schedule reader) input (held reader) ->
        Some reader
    | Nothing | Sole _ | Several -> None
  in
  let is_literal number =
    Ports.node_end ports number = Ports.node_start ports number + 1
    && Decisions.literal (Flows.types flows) (Schedule.node schedule number)
    || Flows.computes_first flows number
  in
  (* A Match computes its output where read only when its branches evaluate nothing but what they give. *)
  let inlining_reader start finish number regions_evaluate_nothing =
    let value = Ports.node_start ports number in
    if
      Ports.node_end ports number = value + 1
      && ((not (Flows.reached flows value)) || (Flows.copied flows value && regions_evaluate_nothing))
      && start < number
      && (not followed.(number))
      && computed_where_read (Schedule.node schedule number)
    then reader_within start finish number value
    else None
  in
  let place_inlined start number position =
    inlined.(number) <- true;
    evaluate start position number;
    false
  in
  let place start finish number position regions_evaluate_nothing =
    if is_literal number then place_inlined start number position
    else
      match inlining_reader start finish number regions_evaluate_nothing with
      | Some reader -> place_inlined start number evaluations.(reader)
      | None ->
          evaluate start position number;
          not (Flows.computes_nothing flows number)
  in
  let rec walk region =
    let start, finish = (Schedule.start schedule region, Schedule.end_ schedule region) in
    visit_from start finish (finish - 1) 0
  and visit_from start finish number evaluated =
    if number < start then evaluated
    else begin
      apart.(number) <- visit start finish number && start < number && number < finish - 1 && not followed.(number);
      visit_from start finish (number - 1) (if apart.(number) then evaluated + 1 else evaluated)
    end
  and visit start finish number =
    definitions.(number) <- take ();
    let node = Schedule.node schedule number in
    if Node.region_node node then place_structured start finish number (Node.regions node)
    else place start finish number definitions.(number) true
  and place_structured start finish number regions =
    let evaluated = List.fold_left (fun total region -> total + walk region) 0 (List.rev regions) in
    place start finish number (take ()) (evaluated = 0)
  in
  ignore (walk (Schedule.body schedule));
  { ports; flows; inlined; apart; last_uses; definitions; following }

let inlined liveness number = liveness.inlined.(number)
let apart liveness number = liveness.apart.(number)
let following liveness number = liveness.following.(number)

let live liveness value =
  Option.is_some liveness.last_uses.(value)
  && not
       liveness.inlined.(Ports.node_of liveness.ports
                           (Ports.port_of liveness.ports (Flows.congruence liveness.flows value)))

let definition liveness value = liveness.definitions.(Ports.node_of liveness.ports (Ports.port_of liveness.ports value))
let last_use liveness value = Option.get liveness.last_uses.(value)
