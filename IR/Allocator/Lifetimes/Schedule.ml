open IR

type 'x t = { body : 'x Region.t; nodes : 'x Node.t Dynarray.t; numbers : ('x, int) NodeTable.t }

let create body =
  let nodes = Dynarray.create () in
  let numbers = NodeTable.create 1024 in
  let place node =
    NodeTable.replace numbers node (Dynarray.length nodes);
    Dynarray.add_last nodes node
  in
  let rec visit node =
    if not (NodeTable.mem numbers node) then begin
      for index = Array.length node.inputs - 1 downto 0 do
        visit node.inputs.(index).node
      done;
      place node
    end
  in
  let rec schedule (region : 'x Region.t) =
    let start = Dynarray.length nodes in
    place region.arguments;
    visit region.results;
    for number = start to Dynarray.length nodes - 1 do
      List.iter schedule (Node.regions (Dynarray.get nodes number))
    done
  in
  schedule body;
  { body; nodes; numbers }

let body schedule = schedule.body
let count schedule = Dynarray.length schedule.nodes
let node schedule number = Dynarray.get schedule.nodes number
let number schedule node = NodeTable.find schedule.numbers node
let start schedule (region : 'x Region.t) = number schedule region.arguments
let end_ schedule (region : 'x Region.t) = number schedule region.results + 1
