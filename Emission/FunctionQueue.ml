open IR

type 'x t = { names : ('x, Name.t) RegionTable.t; pending : ('x Region.t * Name.t) Queue.t }

let create () = { names = RegionTable.create 64; pending = Queue.create () }

let name queue body =
  match RegionTable.find_opt queue.names body with
  | Some name -> name
  | None ->
      let name = Name.Region (RegionTable.length queue.names) in
      RegionTable.replace queue.names body name;
      Queue.push (body, name) queue.pending;
      name

let rec build queue build_function =
  match Queue.take_opt queue.pending with
  | Some (body, name) ->
      let built = build_function body name in
      built :: build queue build_function
  | None -> []
