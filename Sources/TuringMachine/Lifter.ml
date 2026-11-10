open IR
open Definitions

let tape_length = 65536
let import_namespace = "turing"
let carried = [ Type.bytes; Type.i32; Type.unit ]

type 'x machine = { tape : 'x link; pointer : 'x link; reads : 'x link list; input_output_order : 'x link }

let result node = Link.output node 0
let constant value = result (Node.create (I32Constant value) [||])
let widen cell = result (Node.create (Operator (Conversion (Widen `I8))) [| cell |])
let machine_of argument = { tape = argument 0; pointer = argument 1; reads = []; input_output_order = argument 2 }
let tape_after_reads machine = result (Node.create Fence (Array.of_list (machine.tape :: machine.reads)))
let links_of machine = [| tape_after_reads machine; machine.pointer; machine.input_output_order |]

let read_cell types machine =
  let get = result (ArrayGet.create types machine.tape machine.pointer) in
  (get, { machine with reads = get :: machine.reads })

let write_cell types machine value =
  let set = Write.array_set types (tape_after_reads machine) machine.pointer value in
  { machine with tape = result set; reads = [] }

let change_cell types machine change =
  let cell, _ = read_cell types machine in
  let widened = widen cell in
  write_cell types machine (result (Node.create (Operator (Conversion (Narrow `I8))) [| change widened (constant 1) |]))

let move_pointer machine move = { machine with pointer = move machine.pointer (constant 1) }

type 'x host = { types : 'x TypeArena.t; output : Type.t; input : Type.t }

let imported host box_type identifier =
  let import = Node.create (Import { import_type = box_type; namespace = import_namespace; identifier }) [||] in
  result (BoxGet.create host.types (result import))

let output_cell host machine =
  let cell, read = read_cell host.types machine in
  let call = Node.create FunctionApply [| imported host host.output "output"; cell; machine.input_output_order |] in
  { read with input_output_order = result call }

let input_cell host machine =
  let call = Node.create FunctionApply [| imported host host.input "input"; machine.input_output_order |] in
  { (write_cell host.types machine (result call)) with input_output_order = Link.output call 1 }

let test_cell types machine =
  let cell, read = read_cell types machine in
  let nonzero = result (Node.create (Operator (IntegerComparison (`I32, LessThanU))) [| constant 0; widen cell |]) in
  let step = TypeArena.intern types (Sum [ carried; carried ]) in
  result (SumFromI32.create types step (result (Node.create I32FromSum [| nonzero |])) (links_of read))

let binary operator left right = result (Node.create (Operator (IntegerBinary (`I32, operator))) [| left; right |])

let rec lift_instructions host machine instructions = List.fold_left (lift_instruction host) machine instructions

and lift_instruction host machine (instruction : Instruction.t) =
  match instruction with
  | MoveRight -> move_pointer machine (binary Add)
  | MoveLeft -> move_pointer machine (binary Subtract)
  | Increment -> change_cell host.types machine (binary Add)
  | Decrement -> change_cell host.types machine (binary Subtract)
  | Output -> output_cell host machine
  | Input -> input_cell host machine
  | Loop body -> lift_loop host machine body

and lift_repeat host entry body =
  let lift_step links = test_cell host.types (lift_instructions host (machine_of (Array.get links)) body) in
  machine_of (Link.output (RegionRepeat.create host.types (links_of entry) carried lift_step))

and lift_loop host machine body =
  let enter =
    RegionMatch.create host.types (test_cell host.types machine) carried (fun variant fields ->
        let entry = machine_of (Array.get fields) in
        if variant = 1 then links_of (lift_repeat host entry body) else links_of entry)
  in
  machine_of (Link.output enter)

let lift types instructions : 'x CompilationUnit.t =
  let host_function parameters results = TypeArena.intern types (Function (parameters, results)) in
  let output = TypeArena.create_box types (fun _ -> host_function [ Type.i8; Type.unit ] [ Type.unit ]) in
  let input = TypeArena.create_box types (fun _ -> host_function [ Type.unit ] [ Type.i8; Type.unit ]) in
  let host = { types; output; input } in
  let root =
    Region.create types [ Type.unit ] [ Type.unit ] ~build:(fun arguments ->
        let tape = ArrayNew.create types (constant tape_length) (result (Node.create (I8Constant 0) [||])) in
        let initial = [| ArrayNew.or_throw types [||] tape; constant 0; Region.parameter arguments 0 |] in
        [| (lift_instructions host (machine_of (Array.get initial)) instructions).input_output_order |])
  in
  { types; root }
