open IR

type 'definition t = {
  definitions : (Name.t * 'definition) Dynarray.t;
  box_names : (Type.t, Name.t) Hashtbl.t;
  by_definition : ('definition, Name.t) Hashtbl.t;
  describing : (Type.t, unit) Hashtbl.t;
  mutable boxes : int;
}

let create () =
  {
    definitions = Dynarray.create ();
    box_names = Hashtbl.create 16;
    by_definition = Hashtbl.create 16;
    describing = Hashtbl.create 4;
    boxes = 0;
  }

let definitions names = Dynarray.to_list names.definitions

let fresh_name names : Name.t =
  names.boxes <- names.boxes + 1;
  Numbered ("Box_", names.boxes - 1)

let define names name definition =
  if not (Hashtbl.mem names.by_definition definition) then Hashtbl.add names.by_definition definition name;
  Dynarray.add_last names.definitions (name, definition);
  name

let take_fresh_name names box =
  let name = fresh_name names in
  Hashtbl.add names.box_names box name;
  name

let name_by_definition names box definition_of =
  Hashtbl.replace names.describing box ();
  let definition = definition_of box in
  Hashtbl.remove names.describing box;
  let name =
    match (Hashtbl.find_opt names.box_names box, Hashtbl.find_opt names.by_definition definition) with
    | Some own, _ -> define names own definition
    | _, Some existing -> existing
    | None, None -> define names (fresh_name names) definition
  in
  Hashtbl.replace names.box_names box name;
  name

let name names box definition_of =
  match Hashtbl.find_opt names.box_names box with
  | Some name -> name
  | None when Hashtbl.mem names.describing box -> take_fresh_name names box
  | None -> name_by_definition names box definition_of
