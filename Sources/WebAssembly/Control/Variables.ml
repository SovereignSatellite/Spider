open IR

type t = {
  locals : Type.t array;
  result_types : Type.t list;
  thrown : Type.t;
  kinds : Type.t array;
  first_slot : int;
  mutable count : int;
  results : int list;
}

let slot variables height value_type =
  let kind = Option.get (Array.find_index (fun kind -> kind = value_type) variables.kinds) in
  let variable = variables.first_slot + (height * Array.length variables.kinds) + kind in
  variables.count <- max variables.count (variable + 1);
  variable

let create locals result_types references thrown =
  let first_slot = Variable.first_local + Array.length locals in
  let variables =
    {
      locals;
      result_types;
      thrown;
      kinds = Array.of_list ([ Type.i32; Type.i64; Type.f32; Type.f64 ] @ references);
      first_slot;
      count = first_slot;
      results = [];
    }
  in
  let results = List.mapi (slot variables) result_types in
  { variables with results }

let local index = Variable.first_local + index

let type_ variables variable =
  if variable = Variable.x then variables.thrown
  else if variable < Variable.first_local then Type.i32
  else if variable < variables.first_slot then variables.locals.(variable - Variable.first_local)
  else variables.kinds.((variable - variables.first_slot) mod Array.length variables.kinds)
