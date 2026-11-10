open IR

type 'x t = {
  schedule : 'x Schedule.t;
  first_ports : int array;
  first_inputs : int array;
  port_nodes : int array;
  port_starts : int array;
  input_ports : int array;
  leaves : Type.t array;
  value_ports : int array;
}

let link_of schedule port_nodes first_ports port =
  Link.output (Schedule.node schedule port_nodes.(port)) (port - first_ports.(port_nodes.(port)))

let create (layouts : 'x Layouts.t) schedule =
  let types = Layouts.types layouts in
  let count = Schedule.count schedule in
  let first_ports = Array.make (count + 1) 0 in
  let first_inputs = Array.make (count + 1) 0 in
  for number = 0 to count - 1 do
    let node = Schedule.node schedule number in
    first_ports.(number + 1) <- first_ports.(number) + Node.output_count types node;
    first_inputs.(number + 1) <- first_inputs.(number) + Array.length node.inputs
  done;
  let port_nodes = Array.make first_ports.(count) 0 in
  let input_ports = Array.make first_inputs.(count) 0 in
  for number = 0 to count - 1 do
    Array.fill port_nodes first_ports.(number) (first_ports.(number + 1) - first_ports.(number)) number;
    Array.iteri
      (fun index (input : 'x Link.t) ->
        input_ports.(first_inputs.(number) + index) <- first_ports.(Schedule.number schedule input.node) + input.port)
      (Schedule.node schedule number).inputs
  done;
  let held =
    Array.init (Array.length port_nodes) (fun port ->
        Layouts.leaves_of layouts (Link.type_ types (link_of schedule port_nodes first_ports port)))
  in
  let leaves = Array.concat (Array.to_list held) in
  let port_starts = Array.make (Array.length port_nodes + 1) 0 in
  let value_ports = Array.make (Array.length leaves) 0 in
  Array.iteri
    (fun port leaf_types ->
      port_starts.(port + 1) <- port_starts.(port) + Array.length leaf_types;
      Array.fill value_ports port_starts.(port) (Array.length leaf_types) port)
    held;
  { schedule; first_ports; first_inputs; port_nodes; port_starts; input_ports; leaves; value_ports }

let count ports = Array.length ports.leaves
let port_count ports = Array.length ports.port_nodes
let leaf ports value = ports.leaves.(value)
let port ports number port = ports.first_ports.(number) + port
let link ports port = link_of ports.schedule ports.port_nodes ports.first_ports port
let port_of ports value = ports.value_ports.(value)
let node_of ports port = ports.port_nodes.(port)
let input_count ports number = ports.first_inputs.(number + 1) - ports.first_inputs.(number)
let input_port ports number input = ports.input_ports.(ports.first_inputs.(number) + input)
let port_of_link ports (link : 'x Link.t) = ports.first_ports.(Schedule.number ports.schedule link.node) + link.port
let start ports port = ports.port_starts.(port)
let end_ ports port = ports.port_starts.(port + 1)
let node_start ports number = ports.port_starts.(ports.first_ports.(number))
let node_end ports number = ports.port_starts.(ports.first_ports.(number + 1))
let value ports link leaf = ports.port_starts.(port_of_link ports link) + leaf
