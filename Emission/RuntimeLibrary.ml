type helper = { needs : string list; code : string }

let parse marker text =
  let rec header needs start =
    if start + String.length marker <= String.length text && String.sub text start (String.length marker) = marker then
      let named = start + String.length marker in
      let next =
        match String.index_from_opt text named '\n' with Some newline -> newline + 1 | None -> String.length text
      in
      header (String.trim (String.sub text named (next - named)) :: needs) next
    else { needs = List.rev needs; code = String.trim (String.sub text start (String.length text - start)) }
  in
  header [] 0

type t = (string, helper) Hashtbl.t

let resolve runtime needed =
  let emitted = Hashtbl.create 64 in
  let ordered = Dynarray.create () in
  let rec emit name =
    if not (Hashtbl.mem emitted name) then begin
      Hashtbl.add emitted name ();
      let helper = Hashtbl.find runtime name in
      List.iter emit helper.needs;
      Dynarray.add_last ordered helper.code
    end
  in
  List.iter emit (List.sort String.compare needed);
  Dynarray.to_list ordered

let codes runtime names = resolve runtime (Hashtbl.fold (fun name _ names -> name :: names) names [])
let sources runtime = codes runtime runtime

type runtime_library = t

module NeededHelpers = struct
  type t = { runtime : runtime_library; needs : (string, unit) Hashtbl.t }

  let create runtime = { runtime; needs = Hashtbl.create 16 }
  let need needed helper = Hashtbl.replace needed.needs helper ()
  let sources needed = codes needed.runtime needed.needs
end

let of_resources marker resources =
  Hashtbl.of_seq
    (Seq.map
       (fun (path, text) -> (Filename.remove_extension (Filename.basename path), parse marker text))
       (List.to_seq resources))

let helper_name types node = String.concat "" (String.split_on_char '.' (IR.Node.name types node))
