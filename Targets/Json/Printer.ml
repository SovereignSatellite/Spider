open IR
open Definitions

let label types node =
  let name = Node.name types node in
  match node.kind with
  | I8Constant value | I16Constant value | I32Constant value -> Printf.sprintf "%s %Lu" name (Int64.of_int value)
  | I64Constant value -> Printf.sprintf "%s %Lu" name value
  | F32Constant bits -> Printf.sprintf "%s %.9g" name (Int32.float_of_bits bits)
  | F64Constant value -> Printf.sprintf "%s %.17g" name value
  | SumNew { variant; _ } -> Printf.sprintf "%s %d" name variant
  | Import { namespace; identifier; _ } -> Printf.sprintf "%s %s.%s" name namespace identifier
  | Export identifier -> Printf.sprintf "%s %s" name identifier
  | ArrayOf { bytes; _ } -> Printf.sprintf "%s %d bytes" name (String.length bytes)
  | RegionArguments _ | RegionResults _ | RegionReference _ | FunctionApply | FunctionTry _ | FunctionRetype _
  | RegionMatch _ | RegionRepeat _ | Operator _ | Throw _ | SumFromI32 _ | I32FromSum | ArrayNew _ | ArraySize
  | ArrayGet _ | ArraySet _ | ArrayFill _ | ArrayCopy _ | ArraySame | ArrayFinalize _ | BoxNew _ | BoxGet _ | BoxSet _
  | BoxRetype _ | BoxSame | BoxFinalize _ | Fence | Extension _ ->
      name

let function_color = "#8BB1F9"

let node_color node =
  match node.kind with
  | RegionReference _ -> function_color
  | RegionMatch _ -> "#A1FC8F"
  | RegionRepeat _ -> "#E07E7C"
  | RegionArguments _ | RegionResults _ | FunctionApply | FunctionTry _ | FunctionRetype _ | I8Constant _
  | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _ | F64Constant _ | Operator _ | Throw _ | SumNew _
  | SumFromI32 _ | I32FromSum | ArrayNew _ | ArrayOf _ | ArraySize | ArrayGet _ | ArraySet _ | ArrayFill _ | ArrayCopy _
  | ArraySame | ArrayFinalize _ | BoxNew _ | BoxGet _ | BoxSet _ | BoxRetype _ | BoxSame | BoxFinalize _ | Import _
  | Export _ | Fence | Extension _ ->
      "#FFFF93"

type 'x graph = {
  types : 'x type_arena;
  identifiers : ('x, int) NodeTable.t;
  named : 'x RegionSet.t;
  strings : string Dynarray.t;
  string_indexes : (string, int) Hashtbl.t;
  subgraphs : int Dynarray.t;
  nodes : int Dynarray.t;
  edges : int Dynarray.t;
}

let add_string graph text =
  Hashtbl.add graph.string_indexes text (Dynarray.length graph.strings);
  Dynarray.add_last graph.strings text;
  Dynarray.length graph.strings - 1

let intern graph text =
  match Hashtbl.find_opt graph.string_indexes text with Some index -> index | None -> add_string graph text

let rec add_region graph color region =
  add_node graph color region.arguments;
  Array.iter
    (fun node -> if node != region.arguments then add_node graph color node)
    (Passes.RegionNodes.ordered region);
  (NodeTable.find graph.identifiers region.arguments, NodeTable.find graph.identifiers region.results)

and add_node graph boundary_color node =
  let identifier = NodeTable.length graph.identifiers in
  NodeTable.replace graph.identifiers node identifier;
  let color = match node.kind with RegionArguments _ | RegionResults _ -> boundary_color | _ -> node_color node in
  Dynarray.add_last graph.nodes identifier;
  Dynarray.add_last graph.nodes (intern graph (label graph.types node));
  Dynarray.add_last graph.nodes (intern graph color);
  Array.iteri
    (fun index input ->
      Dynarray.add_last graph.edges (NodeTable.find graph.identifiers input.node);
      Dynarray.add_last graph.edges input.port;
      Dynarray.add_last graph.edges identifier;
      Dynarray.add_last graph.edges index)
    node.inputs;
  List.iter (add_subgraph graph node identifier) (Node.regions node);
  match node.kind with
  | RegionReference body when RegionSet.add graph.named body -> add_subgraph graph node identifier body
  | _ -> ()

and add_subgraph graph node identifier region =
  let entry, exit = add_region graph (node_color node) region in
  Dynarray.add_last graph.subgraphs identifier;
  Dynarray.add_last graph.subgraphs entry;
  Dynarray.add_last graph.subgraphs exit

let write_string output text =
  Buffer.add_char output '"';
  String.iter
    (function
      | '"' -> Buffer.add_string output "\\\""
      | '\\' -> Buffer.add_string output "\\\\"
      | character when Char.code character < 0x20 ->
          Buffer.add_string output "\\u00";
          Buffer.add_char output "0123456789abcdef".[Char.code character lsr 4];
          Buffer.add_char output "0123456789abcdef".[Char.code character land 15]
      | character -> Buffer.add_char output character)
    text;
  Buffer.add_char output '"'

let write_array output write items =
  Buffer.add_char output '[';
  Dynarray.iteri
    (fun index item ->
      if index > 0 then Buffer.add_char output ',';
      write output item)
    items;
  Buffer.add_char output ']'

let write_integer output value = Buffer.add_string output (string_of_int value)

let print (compilation_unit : 'x CompilationUnit.t) =
  let graph =
    {
      types = compilation_unit.types;
      identifiers = NodeTable.create 1024;
      named = RegionSet.create 16;
      strings = Dynarray.create ();
      string_indexes = Hashtbl.create 64;
      subgraphs = Dynarray.create ();
      nodes = Dynarray.create ();
      edges = Dynarray.create ();
    }
  in
  ignore (RegionSet.add graph.named compilation_unit.root);
  ignore (add_region graph function_color compilation_unit.root);
  let output = Buffer.create 65536 in
  Buffer.add_string output "{\"subgraphs\":";
  write_array output write_integer graph.subgraphs;
  Buffer.add_string output ",\"nodes\":";
  write_array output write_integer graph.nodes;
  Buffer.add_string output ",\"edges\":";
  write_array output write_integer graph.edges;
  Buffer.add_string output ",\"strings\":";
  write_array output write_string graph.strings;
  Buffer.add_char output '}';
  Buffer.contents output
