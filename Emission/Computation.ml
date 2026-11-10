open IR
open Allocator

type ('x, 'expression) reader = { value : 'x Link.t -> int -> 'expression; test : 'x Link.t -> int -> 'expression }

type ('expression, 'statement) t =
  | Statements of 'statement list
  | Each of (int -> 'expression option)
  | All of 'expression  (** Every leaf in one value, as a call returns them, which one assignment takes apart. *)

let each_leaf value = Each (fun leaf -> Some (value leaf))
let single value = each_leaf (fun _ -> value)

let only_leaf computation =
  match computation with Each leaf_of -> leaf_of 0 | All value -> Some value | Statements _ -> None

let output_leaves (layouts : 'x Layouts.t) node =
  List.concat
    (List.init
       (Node.output_count (Layouts.types layouts) node)
       (fun port ->
         let link = Link.output node port in
         List.init (Layouts.leaf_count layouts (Link.type_ (Layouts.types layouts) link)) (fun leaf -> (link, leaf))))

let leaf_type_of layouts link leaf = (Layouts.leaves_of layouts (Link.type_ (Layouts.types layouts) link)).(leaf)
let label labels comment body = Option.to_list (Option.map comment (RegionTable.find_opt labels body))

let leaf_values layouts reader link =
  List.init (Layouts.leaf_count layouts (Link.type_ (Layouts.types layouts) link)) (reader.value link)

let input_values layouts reader (node : 'x Node.t) first =
  List.concat_map (leaf_values layouts reader) (List.drop first (Array.to_list node.inputs))

let reads_once layouts (node : 'x Node.t) input held =
  let several link = Layouts.leaf_count layouts (Link.type_ (Layouts.types layouts) link) > 1 in
  several node.inputs.(input) || (held <= 1 && not (Node.reads (Layouts.types layouts) node && Node.exists several node))

let each_array layouts array statement = Statements (List.init (Layouts.leaf_count layouts array) statement)

let element_value layouts nothing reader element leaf =
  if Layouts.leaf_count layouts (Link.type_ (Layouts.types layouts) element) > 0 then reader.value element leaf
  else nothing

let leafwise layouts nothing reader (node : 'x Node.t) leaf =
  let count link = Layouts.leaf_count layouts (Link.type_ (Layouts.types layouts) link) in
  let written = count node.inputs.(0) in
  List.map
    (fun input -> element_value layouts nothing reader input (if count input = written then leaf else 0))
    (Array.to_list node.inputs)

let calls (layouts : 'x Layouts.t) nothing text call statement reader (node : 'x Node.t) =
  let call arguments = call (List.map text (Node.strings node.kind) @ arguments) in
  if Node.reads (Layouts.types layouts) node then
    each_array layouts
      (Node.operand_type (Layouts.types layouts) node)
      (fun leaf -> statement (call (leafwise layouts nothing reader node leaf)))
  else All (call (input_values layouts reader node 0))

let create_sum layouts tag sum_type variant =
  let tagged = Layouts.tagged layouts sum_type in
  Each (fun leaf -> if tagged && leaf = 0 then Some (tag variant) else None)

let runs computed =
  List.fold_right
    (fun (variant, value) runs ->
      match runs with
      | (variants, next) :: rest when next = value -> (variant :: variants, next) :: rest
      | _ -> ([ variant ], value) :: runs)
    (List.mapi (fun variant value -> (variant, value)) computed)
    []
