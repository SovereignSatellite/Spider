open Wasm.Source
open Types

(** The reference decoder drops a custom section no handler is registered for; this one keeps the [name] section raw. *)
module Handler = struct
  include Wasm.Handler_custom

  let name = Wasm.Utf8.decode "name"
  let decode _ _ custom = custom
end

let () = Wasm.Custom.register (module Handler)

let rec unsigned bytes position =
  let read = Char.code bytes.[position] in
  if read land 0x80 = 0 then (read, position + 1)
  else
    let rest, next = unsigned bytes (position + 1) in
    ((rest lsl 7) lor (read land 0x7F), next)

(** The reference interpreter's [name] handler keeps the function names it decodes abstract, so subsection 1 is decoded
    here. *)
let read_function_names names section =
  let rec assign remaining position =
    if remaining > 0 then begin
      let index, position = unsigned section position in
      let length, position = unsigned section position in
      Hashtbl.replace names index (String.sub section position length);
      assign (remaining - 1) (position + length)
    end
  in
  let rec subsection position =
    if position < String.length section then
      let size, start = unsigned section (position + 1) in
      if section.[position] = '\001' then
        let count, first = unsigned section start in
        assign count first
      else subsection (start + size)
  in
  subsection 0

let names definition binary customs =
  let names = Hashtbl.create 16 in
  List.find_map
    (fun (module Section : Wasm.Custom.Section) ->
      let custom = (Section.Handler.encode definition binary Section.it).it in
      if custom.name = Handler.name then Some custom.content else None)
    customs
  |> Option.iter (fun section ->
      try read_function_names names section with Invalid_argument _ -> raise (Invalid "malformed name section"));
  names

let label definition binary customs imported_count =
  let names = names definition binary customs in
  let exported_names = Hashtbl.create 16 in
  List.iter
    (fun (export : Wasm.Ast.export) ->
      let (Export (name, exported)) = export.it in
      match exported.it with
      | FuncX function_index -> Hashtbl.replace exported_names (index function_index) (Wasm.Utf8.encode name)
      | _ -> ())
    definition.it.exports;
  fun offset ->
    let index = imported_count + offset in
    match (Hashtbl.find_opt names index, Hashtbl.find_opt exported_names index) with
    | Some name, _ | None, Some name -> Printf.sprintf "%s (function %d)" name index
    | None, None -> Printf.sprintf "function %d" index
