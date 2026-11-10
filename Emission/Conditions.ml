open IR
open Allocator

module type Spelling = sig
  type expression

  val truth : bool -> expression
  val equal : expression -> int -> expression
  val both : expression -> expression -> expression
  val either : expression -> expression -> expression
  val if_else : expression -> expression -> expression -> expression
  val not_ : expression -> expression
end

let constant (computing : 'x Link.t) variant =
  match (computing.node.kind, Node.bits computing.node.kind) with
  | SumNew sum, _ -> Some (sum.variant = variant)
  | _, Some bits -> Some (bits = Int64.of_int variant)
  | _, None -> None

let known allocation tag variant =
  Option.bind (Allocation.inlined allocation tag 0) (fun computing -> constant computing variant)

module type S = sig
  type expression
  type condition = Known of bool | Holds of expression | Is of expression * int

  val holds : expression -> int -> expression
  val render : condition -> expression
  val choose : (int -> condition) -> condition -> condition -> condition

  val reader :
    'x Layouts.t ->
    'x Allocation.t ->
    (expression -> (int list * condition) list -> condition) ->
    (('x, expression) Computation.reader -> 'x Node.t -> int -> expression option) ->
    (('x, expression) Computation.reader -> 'x Node.t -> (expression, 'statement) Computation.t) ->
    (Register.t -> Type.t -> expression) ->
    (Type.t -> expression) ->
    ('x, expression) Computation.reader

  val choose_value :
    'x Layouts.t ->
    (expression -> (int list * expression) list -> expression) ->
    ('x, expression) Computation.reader ->
    'x Node.t ->
    expression
end

module Make (Spelling : Spelling) = struct
  type expression = Spelling.expression
  type condition = Known of bool | Holds of expression | Is of expression * int

  let holds condition variant = if variant = 1 then condition else Spelling.not_ condition

  let render condition =
    match condition with
    | Known truth -> Spelling.truth truth
    | Holds boolean -> boolean
    | Is (value, variant) -> Spelling.equal value variant

  let both first second = Holds (Spelling.both (render first) (render second))
  let either first second = Holds (Spelling.either (render first) (render second))
  let if_else holds on_true on_false = Holds (Spelling.if_else (render holds) (render on_true) (render on_false))

  let choose selected on_false on_true =
    match (selected 1, on_false, on_true) with
    | holds, Known false, Known true -> holds
    | _, Known true, Known false -> selected 0
    | holds, Known false, _ -> both holds on_true
    | _, _, Known false -> both (selected 0) on_false
    | holds, _, Known true -> either holds on_false
    | _, Known true, _ -> either (selected 0) on_true
    | holds, _, _ -> if_else holds on_true on_false

  let compared condition = match condition with Is _ -> true | Known _ | Holds _ -> false

  let test select native allocation (reader : ('x, expression) Computation.reader) tag variant =
    let rec condition tag variant =
      match Allocation.inlined allocation tag 0 with
      | Some computing -> computed tag computing variant
      | None -> Is (reader.value tag 0, variant)
    and computed tag computing variant =
      match (constant computing variant, computing.node.kind) with
      | Some truth, _ -> Known truth
      | None, RegionMatch branches -> chosen tag computing.node branches computing.port variant
      | None, _ -> (
          match native computing.node variant with
          | Some boolean -> Holds boolean
          | None -> Is (reader.value tag 0, variant))
    and chosen tag matcher branches port variant =
      let selector = RegionMatch.selector matcher in
      let branches =
        List.map (fun (branch : 'x Region.t) -> condition branch.results.inputs.(port) variant) (Array.to_list branches)
      in
      match branches with
      | _ when List.for_all compared branches -> Is (reader.value tag 0, variant)
      | [ on_false; on_true ] -> choose (condition selector) on_false on_true
      | _ -> select (reader.value selector 0) (Computation.runs branches)
    in
    render (condition tag variant)

  let reader layouts allocation select native compute read undefined =
    let rec reader : ('x, expression) Computation.reader =
      {
        value =
          (fun link leaf ->
            let computed =
              match (Allocation.inlined allocation link leaf, Allocation.register allocation link leaf) with
              | Some computing, _ -> Computation.only_leaf (compute reader computing.node)
              | None, Some register -> Some (read register (Computation.leaf_type_of layouts link leaf))
              | None, None -> None
            in
            match computed with Some value -> value | None -> undefined (Computation.leaf_type_of layouts link leaf));
        test = (fun sum variant -> test select (native reader) allocation reader sum variant);
      }
    in
    reader

  let choose_value layouts many reader matcher =
    let result (branch : 'x Region.t) =
      match Computation.input_values layouts reader branch.results 0 with
      | [ value ] -> value
      | _ -> invalid_arg "A branch chosen as a value gives one leaf."
    in
    let selector = RegionMatch.selector matcher in
    match Array.to_list (RegionMatch.branches matcher) with
    | [ on_false; on_true ] ->
        let condition = reader.test selector 1 in
        let value = result on_true in
        Spelling.if_else condition value (result on_false)
    | branches ->
        let tag = reader.value selector 0 in
        many tag (Computation.runs (List.map result branches))
end
