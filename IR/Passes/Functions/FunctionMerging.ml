open IR
open Definitions

let optimization = Options.optimization "merge-functions" Function Calls 3 "Merge functions with equal bodies."

let run root functions =
  let merged = RegionTable.create 16 in
  let kept = Hashtbl.create 64 in
  let merge_or_keep bodies body =
    match Dynarray.find_opt (fun earlier -> Equivalence.regions body.types Int.equal earlier body) bodies with
    | Some earlier -> RegionTable.replace merged body earlier
    | None -> Dynarray.add_last bodies body
  in
  List.iter
    (fun body ->
      if body != root then
        let signature = (Region.parameter_types body, Region.result_types body) in
        match Hashtbl.find_opt kept signature with
        | Some bodies -> merge_or_keep bodies body
        | None -> Hashtbl.replace kept signature (Dynarray.of_list [ body ]))
    functions;
  let rename node =
    match node.kind with
    | RegionReference body when RegionTable.mem merged body ->
        Some [| Link.output (Node.create (RegionReference (RegionTable.find merged body)) [||]) 0 |]
    | _ -> None
  in
  Bodies.in_parallel (fun changed body -> ignore (Rewrite.nested rename changed body)) functions
