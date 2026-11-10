open Definitions

let expect types subject expected actual =
  if expected = actual then []
  else
    [
      subject ^ ": expected (" ^ TypeArena.describe_all types expected ^ ") but found ("
      ^ TypeArena.describe_all types actual ^ ")";
    ]

let shaped types subject description fits actual =
  if fits (TypeArena.definition types actual) then []
  else [ subject ^ ": expected " ^ description ^ " but found " ^ TypeArena.describe types actual ]

let array (definition : TypeDefinition.t) = match definition with Structural (Array _) -> true | _ -> false
let box (definition : TypeDefinition.t) = match definition with Box _ -> true | _ -> false
let inputs types node first = List.map (Link.type_ types) (List.drop first (Array.to_list node.inputs))
let region types region = expect types "region results" (Region.result_types region) (inputs types region.results 0)

let matcher types matcher branches selector =
  match TypeArena.definition types selector with
  | Structural (Sum variants) when List.length variants = Array.length branches ->
      List.concat
        (List.mapi
           (fun variant fields ->
             let branch = branches.(variant) and subject = "branch " ^ string_of_int variant in
             expect types (subject ^ " arguments") fields (Node.output_types types branch.arguments)
             @ expect types (subject ^ " results") (Region.result_types branch) (Node.output_types types matcher))
           variants)
  | _ ->
      [
        Printf.sprintf "selector: expected a sum of %d variants but found %s" (Array.length branches)
          (TypeArena.describe types selector);
      ]

let repeat types repeat body =
  let step = Link.type_ types (RegionRepeat.step repeat) in
  expect types "body arguments" (Region.parameter_types body) (inputs types repeat 0)
  @
  match TypeArena.definition types step with
  | Structural (Sum [ exited; continued ]) ->
      expect types "step continue variant" continued (inputs types repeat 0)
      @ expect types "step exit variant" exited (Node.output_types types repeat)
  | _ -> [ "step: expected a sum of two variants but found " ^ TypeArena.describe types step ]

let check types node =
  let input index = Link.type_ types node.inputs.(index) in
  let output () = Node.output_type types node 0 in
  let shaped = shaped types and describe = TypeArena.describe types in
  let array_of subject element =
    shaped subject ("an array of " ^ describe element) (( = ) (TypeDefinition.Structural (Array element)))
  in
  let box_of subject contents =
    shaped subject ("a box of " ^ describe contents) (( = ) (TypeDefinition.Box contents))
  in
  let i32s subject indices = expect types subject (List.map (fun _ -> Type.i32) indices) (List.map input indices) in
  let same kind fits = shaped "left" kind fits (input 0) @ expect types "right" [ input 0 ] [ input 1 ] in
  let retyped kind fits = shaped "input" kind fits (input 0) @ shaped "output" kind fits (output ()) in
  match node.kind with
  | ArrayOf _ ->
      shaped "array" "an array of integers or numbers"
        (function
          | Structural (Array element) -> (
              match TypeArena.definition types element with Structural (Integer _ | Number _) -> true | _ -> false)
          | _ -> false)
        (output ())
  | FunctionApply | FunctionTry _ -> (
      match TypeArena.definition types (input 0) with
      | Structural (Function (parameters, _)) -> expect types "arguments" parameters (inputs types node 1)
      | _ -> [ "callee: expected a function but found " ^ describe (input 0) ])
  | FunctionRetype _ -> retyped "a function" (function Structural (Function _) -> true | _ -> false)
  | RegionMatch branches -> matcher types node branches (input 0)
  | RegionRepeat body -> repeat types node body
  | SumNew { variant; _ } -> (
      match TypeArena.definition types (output ()) with
      | Structural (Sum variants) when variant >= 0 && variant < List.length variants ->
          expect types "fields" (List.nth variants variant) (inputs types node 0)
      | _ -> [ Printf.sprintf "variant: %d does not exist in %s" variant (describe (output ())) ])
  | SumFromI32 _ -> i32s "tag" [ 0 ]
  | ArrayNew _ -> i32s "length" [ 0 ]
  | I32FromSum ->
      shaped "sum" "a sum of two or more variants"
        (function Structural (Sum (_ :: _ :: _)) -> true | _ -> false)
        (input 0)
  | ArraySize | ArrayFinalize _ -> shaped "array" "an array" array (input 0)
  | ArrayGet _ -> array_of "array" (output ()) (input 0) @ i32s "index" [ 1 ]
  | ArraySet _ -> array_of "array" (input 2) (input 0) @ i32s "index" [ 1 ]
  | ArrayFill _ -> array_of "array" (input 3) (input 0) @ i32s "range" [ 1; 2 ]
  | ArrayCopy _ ->
      shaped "destination" "an array" array (input 0)
      @ expect types "source" [ input 0 ] [ input 2 ]
      @ i32s "range" [ 1; 3; 4 ]
  | ArraySame -> same "an array" array
  | BoxNew _ -> box_of "box" (input 0) (output ())
  | BoxGet _ -> box_of "box" (output ()) (input 0)
  | BoxSet _ -> box_of "box" (input 1) (input 0)
  | BoxRetype _ -> retyped "a box" box
  | BoxSame -> same "a box" box
  | BoxFinalize _ -> shaped "box" "a box" box (input 0)
  | Import _ -> shaped "import" "a box" box (output ())
  | Export _ -> shaped "export" "a box" box (input 0)
  | RegionArguments _ | RegionResults _ | Fence | Throw _ -> []
  | RegionReference _ | I8Constant _ | I16Constant _ | I32Constant _ | I64Constant _ | F32Constant _ | F64Constant _
  | Operator _ | Extension _ ->
      expect types "inputs" (List.init (Array.length node.inputs) (Node.input_type types node)) (inputs types node 0)
