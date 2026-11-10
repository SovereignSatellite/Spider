open Definitions

type 'x t = 'x node

let next = Atomic.make 0
let create kind inputs = { id = Atomic.fetch_and_add next 1; kind; inputs }
let inputs node = Array.copy node.inputs
let exists predicate node = Array.exists predicate node.inputs
let for_all predicate node = Array.for_all predicate node.inputs

let reading inputs node =
  Array.blit inputs 0 node.inputs 0 (Array.length inputs);
  node

let name types node =
  match node.kind with
  | RegionArguments _ -> "Region.Arguments"
  | RegionResults _ -> "Region.Results"
  | RegionReference _ -> "Region.Reference"
  | FunctionApply -> "Function.Apply"
  | FunctionTry _ -> "Function.Try"
  | FunctionRetype _ -> "Function.Retype"
  | RegionMatch _ -> "Region.Match"
  | RegionRepeat _ -> "Region.Repeat"
  | I8Constant _ -> "I8.Constant"
  | I16Constant _ -> "I16.Constant"
  | I32Constant _ -> "I32.Constant"
  | I64Constant _ -> "I64.Constant"
  | F32Constant _ -> "F32.Constant"
  | F64Constant _ -> "F64.Constant"
  | Operator operator -> Operator.name operator
  | Throw _ -> "Throw"
  | SumNew _ -> "Sum.New"
  | SumFromI32 _ -> "Sum.FromI32"
  | I32FromSum -> "I32.FromSum"
  | ArrayNew _ -> "Array.New"
  | ArrayOf _ -> "Array.Of"
  | ArraySize -> "Array.Size"
  | ArrayGet _ -> "Array.Get"
  | ArraySet _ -> "Array.Set"
  | ArrayFill _ -> "Array.Fill"
  | ArrayCopy _ -> "Array.Copy"
  | ArraySame -> "Array.Same"
  | ArrayFinalize _ -> "Array.Finalize"
  | BoxNew _ -> "Box.New"
  | BoxGet _ -> "Box.Get"
  | BoxSet _ -> "Box.Set"
  | BoxRetype _ -> "Box.Retype"
  | BoxSame -> "Box.Same"
  | BoxFinalize _ -> "Box.Finalize"
  | Import _ -> "Import"
  | Export _ -> "Export"
  | Fence -> "Fence"
  | Extension extended -> types.extension.name extended

let same_kind types left right = String.equal (name types left) (name types right)

let bits kind =
  match kind with
  | I8Constant value | I16Constant value | I32Constant value -> Some (Int64.of_int value)
  | I64Constant value -> Some value
  | F32Constant value -> Some (Int64.logand (Int64.of_int32 value) 0xFFFF_FFFFL)
  | F64Constant value -> Some (Int64.bits_of_float value)
  | RegionArguments _ | RegionResults _ | RegionReference _ | FunctionApply | FunctionTry _ | FunctionRetype _
  | RegionMatch _ | RegionRepeat _ | Operator _ | Throw _ | SumNew _ | SumFromI32 _ | I32FromSum | ArrayNew _
  | ArrayOf _ | ArraySize | ArrayGet _ | ArraySet _ | ArrayFill _ | ArrayCopy _ | ArraySame | ArrayFinalize _ | BoxNew _
  | BoxGet _ | BoxSet _ | BoxRetype _ | BoxSame | BoxFinalize _ | Import _ | Export _ | Fence | Extension _ ->
      None

let strings kind =
  match kind with
  | Import { namespace; identifier; _ } -> [ namespace; identifier ]
  | Export identifier -> [ identifier ]
  | RegionArguments _ | RegionResults _ | RegionReference _ | FunctionApply | FunctionTry _ | FunctionRetype _
  | RegionMatch _ | RegionRepeat _ | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _
  | F64Constant _ | Operator _ | Throw _ | SumNew _ | SumFromI32 _ | I32FromSum | ArrayNew _ | ArrayOf _ | ArraySize
  | ArrayGet _ | ArraySet _ | ArrayFill _ | ArrayCopy _ | ArraySame | ArrayFinalize _ | BoxNew _ | BoxGet _ | BoxSet _
  | BoxRetype _ | BoxSame | BoxFinalize _ | Fence | Extension _ ->
      []

let base types node =
  match node.kind with
  | Extension extended -> Some (types.extension.base extended)
  | ArraySet _ | ArrayFill _ | ArrayCopy _ | BoxSet _ -> Some Write
  | Operator operator -> Some (if Operator.binary operator then BinaryOperation else UnaryOperation)
  | RegionArguments _ | RegionResults _ | RegionReference _ | FunctionApply | FunctionTry _ | FunctionRetype _
  | RegionMatch _ | RegionRepeat _ | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _
  | F64Constant _ | Throw _ | SumNew _ | SumFromI32 _ | I32FromSum | ArrayNew _ | ArrayOf _ | ArraySize | ArrayGet _
  | ArraySame | ArrayFinalize _ | BoxNew _ | BoxGet _ | BoxRetype _ | BoxSame | BoxFinalize _ | Import _ | Export _
  | Fence ->
      None

let binary types node = base types node = Some BinaryOperation

let unary types node =
  match base types node with
  | Some (UnaryOperation | ViewOperation) -> true
  | Some (Operation | BinaryOperation | Write) | None -> false

let write types node = base types node = Some Write

let commutative types node =
  match node.kind with
  | Extension extended -> types.extension.commutative extended
  | Operator operator -> Operator.commutative operator
  | RegionArguments _ | RegionResults _ | RegionReference _ | FunctionApply | FunctionTry _ | FunctionRetype _
  | RegionMatch _ | RegionRepeat _ | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _
  | F64Constant _ | Throw _ | SumNew _ | SumFromI32 _ | I32FromSum | ArrayNew _ | ArrayOf _ | ArraySize | ArrayGet _
  | ArraySet _ | ArrayFill _ | ArrayCopy _ | ArraySame | ArrayFinalize _ | BoxNew _ | BoxGet _ | BoxSet _ | BoxRetype _
  | BoxSame | BoxFinalize _ | Import _ | Export _ | Fence ->
      false

let holds types node =
  match node.kind with
  | Extension extended -> types.extension.holds extended
  | Operator operator -> Operator.holds operator
  | RegionArguments _ | RegionResults _ | RegionReference _ | FunctionApply | FunctionTry _ | FunctionRetype _
  | RegionMatch _ | RegionRepeat _ | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _
  | F64Constant _ | Throw _ | SumNew _ | SumFromI32 _ | I32FromSum | ArrayNew _ | ArrayOf _ | ArraySize | ArrayGet _
  | ArraySet _ | ArrayFill _ | ArrayCopy _ | ArraySame | ArrayFinalize _ | BoxNew _ | BoxGet _ | BoxSet _ | BoxRetype _
  | BoxSame | BoxFinalize _ | Import _ | Export _ | Fence ->
      0

let structural node =
  match node.kind with
  | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _ | F64Constant _ -> true
  | RegionArguments _ | RegionResults _ | RegionReference _ | RegionMatch _ | RegionRepeat _ -> true
  | Throw _ | SumNew _ | SumFromI32 _ | I32FromSum | Fence -> true
  | FunctionApply | FunctionTry _ | FunctionRetype _ | Operator _ | ArrayNew _ | ArrayOf _ | ArraySize | ArrayGet _
  | ArraySet _ | ArrayFill _ | ArrayCopy _ | ArraySame | ArrayFinalize _ | BoxNew _ | BoxGet _ | BoxSet _ | BoxRetype _
  | BoxSame | BoxFinalize _ | Import _ | Export _ | Extension _ ->
      false

let unrepeatable types node =
  match node.kind with
  | FunctionApply | FunctionTry _ | ArrayNew _ | ArrayOf _ | ArrayCopy _ | ArrayFinalize _ -> true
  | BoxNew _ | BoxFinalize _ | Import _ | Export _ -> true
  | Extension extended -> types.extension.unrepeatable extended
  | RegionArguments _ | RegionResults _ | RegionReference _ | FunctionRetype _ | RegionMatch _ | RegionRepeat _
  | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _ | F64Constant _ | Operator _ | Throw _
  | SumNew _ | SumFromI32 _ | I32FromSum | ArraySize | ArrayGet _ | ArraySet _ | ArrayFill _ | ArraySame | BoxGet _
  | BoxSet _ | BoxRetype _ | BoxSame | Fence ->
      false

let array_access types node =
  match node.kind with
  | ArrayGet _ | ArraySet _ -> Some { offset = 0; width = 1 }
  | Extension extended -> types.extension.array_access extended
  | RegionArguments _ | RegionResults _ | RegionReference _ | FunctionApply | FunctionTry _ | FunctionRetype _
  | RegionMatch _ | RegionRepeat _ | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _
  | F64Constant _ | Operator _ | Throw _ | SumNew _ | SumFromI32 _ | I32FromSum | ArrayNew _ | ArrayOf _ | ArraySize
  | ArrayFill _ | ArrayCopy _ | ArraySame | ArrayFinalize _ | BoxNew _ | BoxGet _ | BoxSet _ | BoxRetype _ | BoxSame
  | BoxFinalize _ | Import _ | Export _ | Fence ->
      None

let regions node =
  match node.kind with
  | RegionMatch branches -> Array.to_list branches
  | RegionRepeat body -> [ body ]
  | RegionArguments _ | RegionResults _ | RegionReference _ | FunctionApply | FunctionTry _ | FunctionRetype _
  | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _ | F64Constant _ | Operator _ | Throw _
  | SumNew _ | SumFromI32 _ | I32FromSum | ArrayNew _ | ArrayOf _ | ArraySize | ArrayGet _ | ArraySet _ | ArrayFill _
  | ArrayCopy _ | ArraySame | ArrayFinalize _ | BoxNew _ | BoxGet _ | BoxSet _ | BoxRetype _ | BoxSame | BoxFinalize _
  | Import _ | Export _ | Fence | Extension _ ->
      []

let region_node node =
  match node.kind with
  | RegionMatch _ | RegionRepeat _ -> true
  | RegionArguments _ | RegionResults _ | RegionReference _ | FunctionApply | FunctionTry _ | FunctionRetype _
  | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _ | F64Constant _ | Operator _ | Throw _
  | SumNew _ | SumFromI32 _ | I32FromSum | ArrayNew _ | ArrayOf _ | ArraySize | ArrayGet _ | ArraySet _ | ArrayFill _
  | ArrayCopy _ | ArraySame | ArrayFinalize _ | BoxNew _ | BoxGet _ | BoxSet _ | BoxRetype _ | BoxSame | BoxFinalize _
  | Import _ | Export _ | Fence | Extension _ ->
      false

let exits types body =
  match TypeArena.definition types body.result_types.(0) with
  | Structural (Sum (exited :: _)) -> exited
  | _ -> invalid_arg "A Repeat's step is a sum of its exits and its carried values."

let rec output_count types node =
  match node.kind with
  | RegionArguments region -> Array.length region.parameter_types
  | RegionResults _ -> 0
  | FunctionApply -> List.length (snd (TypeArena.signature types (link_type types node.inputs.(0))))
  | RegionMatch branches -> if Array.length branches = 0 then 0 else Array.length branches.(0).result_types
  | RegionRepeat body -> List.length (exits types body)
  | Throw result_types -> Dynarray.length result_types
  | Extension extended -> types.extension.output_count types node extended
  | RegionReference _ | FunctionTry _ | FunctionRetype _ | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _
  | F32Constant _ | F64Constant _ | Operator _ | SumNew _ | SumFromI32 _ | I32FromSum | ArrayNew _ | ArrayOf _
  | ArraySize | ArrayGet _ | ArraySet _ | ArrayFill _ | ArrayCopy _ | ArraySame | ArrayFinalize _ | BoxNew _ | BoxGet _
  | BoxSet _ | BoxRetype _ | BoxSame | BoxFinalize _ | Import _ | Export _ | Fence ->
      1

and output_type types node port =
  match node.kind with
  | RegionArguments region -> region.parameter_types.(port)
  | RegionResults _ -> invalid_arg "A region's results have no outputs."
  | RegionReference body -> body.region_type
  | FunctionApply -> List.nth (snd (TypeArena.signature types (link_type types node.inputs.(0)))) port
  | FunctionTry payload_types ->
      TypeArena.intern types (Sum [ payload_types; snd (TypeArena.signature types (link_type types node.inputs.(0))) ])
  | FunctionRetype function_type -> function_type
  | RegionMatch branches -> branches.(0).result_types.(port)
  | RegionRepeat body -> List.nth (exits types body) port
  | I8Constant _ -> Type.i8
  | I16Constant _ -> Type.i16
  | I32Constant _ | I32FromSum | ArraySize -> Type.i32
  | I64Constant _ -> Type.i64
  | F32Constant _ -> Type.f32
  | F64Constant _ -> Type.f64
  | Throw result_types -> Dynarray.get result_types port
  | SumNew { sum_type; _ } | SumFromI32 { sum_type; _ } -> sum_type
  | ArrayNew { allocated; _ } -> allocated
  | ArrayOf { array_type; _ } -> array_type
  | ArrayGet element_type -> element_type
  | Fence -> link_type types node.inputs.(0)
  | ArraySame | BoxSame -> Type.boolean
  | ArraySet value_type
  | ArrayFill value_type
  | ArrayCopy value_type
  | ArrayFinalize value_type
  | BoxNew value_type
  | BoxGet value_type
  | BoxSet value_type
  | BoxRetype value_type
  | BoxFinalize value_type ->
      value_type
  | Import { import_type; _ } -> import_type
  | Export _ -> Type.unit
  | Extension extended -> types.extension.output_type types node extended port
  | Operator operator -> Operator.result_type operator

and link_type types link = output_type types link.node link.port

let operand_type types node = link_type types node.inputs.(0)

let admitted types node takes otherwise =
  let operand_type = operand_type types node in
  if takes operand_type then operand_type else otherwise

let output_types types node = List.init (output_count types node) (output_type types node)
let outputs types node = Array.init (output_count types node) (fun port -> { node; port })

let operation node =
  match node.kind with
  | Extension _ | Import _ | RegionReference _ | ArrayOf _ | Operator _ -> true
  | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _ | F64Constant _ -> true
  | RegionArguments _ | RegionResults _ | FunctionApply | FunctionTry _ | FunctionRetype _ | RegionMatch _
  | RegionRepeat _ | Throw _ | SumNew _ | SumFromI32 _ | I32FromSum | ArrayNew _ | ArraySize | ArrayGet _ | ArraySet _
  | ArrayFill _ | ArrayCopy _ | ArraySame | ArrayFinalize _ | BoxNew _ | BoxGet _ | BoxSet _ | BoxRetype _ | BoxSame
  | BoxFinalize _ | Export _ | Fence ->
      false

let input_type types node index =
  match node.kind with
  | Extension extended -> types.extension.input_type types node extended index
  | Operator operator -> Type.of_scalar (Operator.operand operator)
  | _ -> invalid_arg (name types node ^ " has no operand type.")

let match_flows node branches visit =
  Array.iteri
    (fun variant branch ->
      for field = 0 to Array.length branch.parameter_types - 1 do
        visit
          {
            (Flow.whole node 0 { node = branch.arguments; port = field }) with
            input_path = VariantField (variant, field);
          }
      done;
      for index = 0 to Array.length branch.result_types - 1 do
        visit (Flow.whole branch.results index { node; port = index })
      done)
    branches

let repeat_flows types node body visit =
  for index = 0 to Array.length node.inputs - 1 do
    visit (Flow.whole node index { node = body.arguments; port = index });
    visit
      { (Flow.whole body.results 0 { node = body.arguments; port = index }) with input_path = VariantField (1, index) }
  done;
  List.iteri
    (fun index _ ->
      visit { (Flow.whole body.results 0 { node; port = index }) with input_path = VariantField (0, index) })
    (exits types body)

let sum_flows node variants tagged visit =
  let flow input path = visit { (Flow.whole node input { node; port = 0 }) with output_path = path } in
  if tagged then flow 0 Tag;
  match variants with
  | variant :: _ ->
      for index = 1 to Array.length node.inputs - 1 do
        flow index (VariantField (variant, index - 1))
      done
  | [] -> ()

let forwarding types node =
  match base types node with
  | Some (ViewOperation | Write) -> true
  | Some (Operation | UnaryOperation | BinaryOperation) | None -> (
      match node.kind with
      | RegionMatch _ | RegionRepeat _ | SumNew _ | SumFromI32 _ | I32FromSum | ArrayFinalize _ | BoxFinalize _ | Fence
        ->
          true
      | RegionArguments _ | RegionResults _ | RegionReference _ | FunctionApply | FunctionTry _ | FunctionRetype _
      | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _ | F64Constant _ | Operator _
      | Throw _ | ArrayNew _ | ArrayOf _ | ArraySize | ArrayGet _ | ArraySet _ | ArrayFill _ | ArrayCopy _ | ArraySame
      | BoxNew _ | BoxGet _ | BoxSet _ | BoxRetype _ | BoxSame | Import _ | Export _ | Extension _ ->
          false)

let flows types node visit =
  match node.kind with
  | RegionMatch branches -> match_flows node branches visit
  | RegionRepeat body -> repeat_flows types node body visit
  | SumNew { variant; _ } ->
      Array.iteri
        (fun index _ ->
          visit { (Flow.whole node index { node; port = 0 }) with output_path = VariantField (variant, index) })
        node.inputs
  | SumFromI32 { variants; tagged; _ } -> sum_flows node variants tagged visit
  | I32FromSum -> visit { (Flow.whole node 0 { node; port = 0 }) with input_path = Tag }
  | ArrayFinalize _ | BoxFinalize _ -> visit (Flow.written node)
  | Fence -> visit (Flow.whole node 0 { node; port = 0 })
  | RegionArguments _ | RegionResults _ | RegionReference _ | FunctionApply | FunctionTry _ | FunctionRetype _
  | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _ | F64Constant _ | Operator _ | Throw _
  | ArrayNew _ | ArrayOf _ | ArraySize | ArrayGet _ | ArraySet _ | ArrayFill _ | ArrayCopy _ | ArraySame | BoxNew _
  | BoxGet _ | BoxSet _ | BoxRetype _ | BoxSame | Import _ | Export _ | Extension _ -> (
      match base types node with
      | Some Write -> visit (Flow.written node)
      | Some ViewOperation -> visit (Flow.whole node 0 { node; port = 0 })
      | Some (Operation | UnaryOperation | BinaryOperation) | None -> ())

let reads types node =
  match node.kind with
  | ArrayFinalize _ | BoxFinalize _ -> true
  | RegionArguments _ | RegionResults _ | RegionReference _ | FunctionApply | FunctionTry _ | FunctionRetype _
  | RegionMatch _ | RegionRepeat _ | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _
  | F64Constant _ | Operator _ | Throw _ | SumNew _ | SumFromI32 _ | I32FromSum | ArrayNew _ | ArrayOf _ | ArraySize
  | ArrayGet _ | ArraySet _ | ArrayFill _ | ArrayCopy _ | ArraySame | BoxNew _ | BoxGet _ | BoxSet _ | BoxRetype _
  | BoxSame | Import _ | Export _ | Fence | Extension _ ->
      write types node
