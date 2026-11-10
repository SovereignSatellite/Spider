open IR

type layout = { leaves : Type.t array; fields : int array array array; whole : int array }
type 'x t = { types : 'x TypeArena.t; cache : (Type.t, layout) Hashtbl.t }

let tag_leaf = [| 0 |]
let create types = { types; cache = Hashtbl.create 64 }
let types layouts = layouts.types
let layout leaves fields = { leaves; fields; whole = Array.init (Array.length leaves) Fun.id }
let tagged_variants variants = List.length variants >= 2
let scalar leaf = layout [| leaf |] [||]

let rec layout_of layouts decomposed =
  match Hashtbl.find_opt layouts.cache decomposed with
  | Some existing -> existing
  | None ->
      let created = compute layouts decomposed in
      Hashtbl.add layouts.cache decomposed created;
      created

and compute layouts decomposed =
  match TypeArena.definition layouts.types decomposed with
  | Structural (Sum variants) -> sum layouts variants
  | Structural (Array element) -> array layouts decomposed element
  | Structural (Integer _ | Number _ | Function _ | Opaque) | Box _ | Extension _ -> scalar decomposed

and array layouts decomposed element =
  match (layout_of layouts element).leaves with
  | [||] -> scalar decomposed
  | leaves -> layout (Array.map (fun leaf -> TypeArena.intern layouts.types (Array leaf)) leaves) [||]

and sum layouts variants =
  let tag = if tagged_variants variants then [| Type.i32 |] else [||] in
  let slots = Dynarray.create () in
  let slot_of = Hashtbl.create 8 in
  let slot_for taken leaf =
    let occurrence = Option.value (Hashtbl.find_opt taken leaf) ~default:0 in
    Hashtbl.replace taken leaf (occurrence + 1);
    if not (Hashtbl.mem slot_of (leaf, occurrence)) then begin
      Hashtbl.add slot_of (leaf, occurrence) (Array.length tag + Dynarray.length slots);
      Dynarray.add_last slots leaf
    end;
    Hashtbl.find slot_of (leaf, occurrence)
  in
  let variant_fields field_types =
    let taken = Hashtbl.create 8 in
    Array.of_list (List.map (fun field -> Array.map (slot_for taken) (layout_of layouts field).leaves) field_types)
  in
  let fields = Array.of_list (List.map variant_fields variants) in
  layout (Array.append tag (Dynarray.to_array slots)) fields

let leaves_of layouts decomposed = (layout_of layouts decomposed).leaves
let leaf_count layouts value_type = Array.length (layout_of layouts value_type).leaves

let tagged layouts sum =
  match TypeArena.definition layouts.types sum with
  | Structural (Sum variants) -> tagged_variants variants
  | Structural (Integer _ | Number _ | Function _ | Array _ | Opaque) | Box _ | Extension _ -> false

let leaves layouts value_types =
  List.concat_map (fun value_type -> Array.to_list (layout_of layouts value_type).leaves) value_types

let select layouts selected (path : Path.t) =
  match path with
  | Whole -> (layout_of layouts selected).whole
  | VariantField (variant, field) -> (layout_of layouts selected).fields.(variant).(field)
  | Tag -> tag_leaf
