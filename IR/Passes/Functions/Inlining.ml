open IR
open Definitions

let optimization =
  Options.optimization "inline-functions" Function Calls 3 "Inline a function that is small or called once."

let small_size = 32
let once_size = 512

type 'x body = { named : 'x region Dynarray.t; mutable size : int }

let describe region =
  let body = { named = Dynarray.create (); size = 0 } in
  let rec within region =
    Array.iter
      (fun node ->
        body.size <- body.size + 1;
        Array.iter
          (fun input ->
            match input.node.kind with RegionReference named -> Dynarray.add_last body.named named | _ -> ())
          node.inputs;
        List.iter within (Node.regions node))
      (RegionNodes.ordered region)
  in
  within region;
  body

let inline_apply inlined node =
  match node.inputs.(0).node.kind with
  | RegionReference body when RegionSet.mem inlined body ->
      Some (Copying.results body (Array.sub node.inputs 1 (Array.length node.inputs - 1)))
  | _ -> None

let inline_call inlined node = match node.kind with FunctionApply -> inline_apply inlined node | _ -> None

let run functions =
  let bodies = RegionTable.create 64 in
  List.iter (fun body -> RegionTable.replace bodies body (describe body)) functions;
  let named body = (RegionTable.find bodies body).named in
  let counts = RegionTable.create 64 in
  List.iter
    (fun body ->
      Dynarray.iter
        (fun callee ->
          RegionTable.replace counts callee (1 + Option.value (RegionTable.find_opt counts callee) ~default:0))
        (named body))
    functions;
  let named_once body = RegionTable.find_opt counts body = Some 1 in
  let reaches_itself body =
    let visited = RegionSet.create 16 in
    let rec reaches from =
      Dynarray.exists (fun callee -> callee == body || (RegionSet.add visited callee && reaches callee)) (named from)
    in
    reaches body
  in
  let settled = RegionSet.create 64 in
  let inlined = RegionSet.create 64 in
  let changed = RegionSet.create 64 in
  let rec settle body =
    if RegionSet.add settled body then begin
      Dynarray.iter settle (named body);
      if Dynarray.exists (RegionSet.mem inlined) (named body) && Rewrite.nested (inline_call inlined) changed body then
        RegionTable.replace bodies body (describe body);
      let size = (RegionTable.find bodies body).size in
      if (size <= small_size || (named_once body && size <= once_size)) && not (reaches_itself body) then
        ignore (RegionSet.add inlined body)
    end
  in
  List.iter settle functions;
  changed
