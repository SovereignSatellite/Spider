type t =
  | IntegerBinary of Scalar.integer * IntegerBinary.t
  | IntegerCount of Scalar.integer * IntegerCount.t
  | IntegerComparison of Scalar.integer * IntegerComparison.t
  | NumberBinary of Scalar.number * NumberBinary.t
  | NumberUnary of Scalar.number * NumberUnary.t
  | NumberComparison of Scalar.number * NumberComparison.t
  | Conversion of Conversion.t

let name operator =
  let named width operation = Scalar.name width ^ "." ^ operation in
  match operator with
  | IntegerBinary (width, operator) -> named width (IntegerBinary.name operator)
  | IntegerCount (width, operator) -> named width (IntegerCount.name operator)
  | IntegerComparison (width, operator) -> named width (IntegerComparison.name operator)
  | NumberBinary (width, operator) -> named width (NumberBinary.name operator)
  | NumberUnary (width, operator) -> named width (NumberUnary.name operator)
  | NumberComparison (width, operator) -> named width (NumberComparison.name operator)
  | Conversion conversion -> named (Conversion.source conversion) (Conversion.name conversion)

let operand operator =
  match operator with
  | IntegerBinary (width, _) | IntegerCount (width, _) | IntegerComparison (width, _) -> (width :> Scalar.t)
  | NumberBinary (width, _) | NumberUnary (width, _) | NumberComparison (width, _) -> (width :> Scalar.t)
  | Conversion conversion -> Conversion.source conversion

let result_type operator =
  match operator with
  | IntegerBinary (width, _) | IntegerCount (width, _) -> Type.of_scalar width
  | NumberBinary (width, _) | NumberUnary (width, _) -> Type.of_scalar width
  | Conversion conversion -> Type.of_scalar (Conversion.target conversion)
  | IntegerComparison _ | NumberComparison _ -> Type.boolean

let binary operator =
  match operator with
  | IntegerBinary _ | IntegerComparison _ | NumberBinary _ | NumberComparison _ -> true
  | IntegerCount _ | NumberUnary _ | Conversion _ -> false

let commutative operator =
  match operator with
  | IntegerBinary (_, operator) -> IntegerBinary.commutative operator
  | IntegerComparison (_, operator) -> IntegerComparison.commutative operator
  | NumberBinary (_, operator) -> NumberBinary.commutative operator
  | NumberComparison (_, operator) -> NumberComparison.commutative operator
  | IntegerCount _ | NumberUnary _ | Conversion _ -> false

let holds operator =
  match operator with
  | IntegerComparison (_, operator) -> IntegerComparison.holds operator
  | NumberComparison (_, operator) -> NumberComparison.holds operator
  | IntegerBinary _ | IntegerCount _ | NumberBinary _ | NumberUnary _ | Conversion _ -> 0
