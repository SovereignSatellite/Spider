open IR
open Definitions
open Types
open Entities
open Traps

let unary kind operands = make kind [| List.hd operands |]
let binary kind operands = make kind (Array.of_list operands)
let swapped kind operands = make kind (Array.of_list (List.rev operands))
let holds comparison = make I32FromSum [| comparison |]
let fails comparison = make (Operator (IntegerBinary (`I32, ExclusiveOr))) [| holds comparison; i32 1 |]
let constant width value = match width with `I32 -> i32 (Int64.to_int value) | `I64 -> make (I64Constant value) [||]
let equal width left right = holds (make (Operator (IntegerComparison (width, Equal))) [| left; right |])

let trapping context result_type tag variant operands compute =
  let sum = select context.types 2 tag (Array.of_list operands) in
  result (traps_in context sum variant [ result_type ] (fun fields -> [| compute (Array.to_list fields) |]))

let integer_binary (operator : Wasm.Ast.IntOp.binop) : IntegerBinary.t =
  match operator with
  | Add -> Add
  | Sub -> Subtract
  | Mul -> Multiply
  | And -> And
  | Or -> Or
  | Xor -> ExclusiveOr
  | Shl -> ShiftLeft
  | Shr S -> ShiftRightS
  | Shr U -> ShiftRightU
  | Rotl -> RotateLeft
  | Rotr -> RotateRight
  | Div S -> DivideS
  | Div U -> DivideU
  | Rem S -> RemainderS
  | Rem U -> RemainderU

let division context width (operator : Wasm.Ast.IntOp.binop) operands =
  let left = List.nth operands 0 in
  let right = List.nth operands 1 in
  let by_zero = equal width right (constant width 0L) in
  let least = Int64.shift_left 1L (Scalar.bits width - 1) in
  let traps =
    match operator with
    | Div S ->
        let overflow =
          make
            (Operator (IntegerBinary (`I32, And)))
            [| equal width left (constant width least); equal width right (constant width (-1L)) |]
        in
        make (Operator (IntegerBinary (`I32, Or))) [| by_zero; overflow |]
    | _ -> by_zero
  in
  trapping context (Type.of_scalar width) traps 1 operands
    (binary (Operator (IntegerBinary (width, integer_binary operator))))

let saturate target (signedness : Wasm.Ast.IntOp.sx) : Conversion.t =
  match signedness with S -> SaturateSigned target | U -> SaturateUnsigned target

(** Inside the open bounds the saturating truncation equals the trapping one, and NaN fails both comparisons. *)
let truncation context target lower upper saturate source operands =
  let operand = source (List.hd operands) in
  let above = make (Operator (NumberComparison (`F64, LessThan))) [| make (F64Constant lower) [||]; operand |] in
  let below = make (Operator (NumberComparison (`F64, LessThan))) [| operand; make (F64Constant upper) [||] |] in
  let inside = make (Operator (IntegerBinary (`I32, And))) [| holds above; holds below |] in
  trapping context (Type.of_scalar target) inside 0 [ operand ] (unary (Operator (Conversion saturate)))

let integer_traps context target (operator : Wasm.Ast.IntOp.cvtop) =
  let bounded source (signedness : Wasm.Ast.IntOp.sx) =
    let lower, upper =
      match (target, signedness) with
      | `I32, S -> (-2147483649.0, 2147483648.0)
      | `I32, U -> (-1.0, 4294967296.0)
      | `I64, S -> (-9223372036854777856.0, 9223372036854775808.0)
      | `I64, U -> (-1.0, 18446744073709551616.0)
    in
    Some (truncation context target lower upper (saturate target signedness) source)
  in
  match operator with
  | TruncF32 signedness -> bounded (converted (Widen `F32)) signedness
  | TruncF64 signedness -> bounded Fun.id signedness
  | ExtendI32 _ | WrapI64 | TruncSatF32 _ | TruncSatF64 _ | ReinterpretFloat -> None

let traps_on context (instruction : Wasm.Ast.instr') =
  match instruction with
  | Binary (I32 ((Div _ | Rem _) as operator)) -> Some (division context `I32 operator)
  | Binary (I64 ((Div _ | Rem _) as operator)) -> Some (division context `I64 operator)
  | Convert (I32 operator) -> integer_traps context `I32 operator
  | Convert (I64 operator) -> integer_traps context `I64 operator
  | _ -> None

let signed64 operand = converted (SignExtend (`I64, `I32)) (converted (Widen `I32) operand)

let integer_comparison width (operator : Wasm.Ast.IntOp.relop) operands =
  let compared comparison = Operator (IntegerComparison (width, comparison)) in
  match operator with
  | Eq -> holds (binary (compared Equal) operands)
  | Ne -> fails (binary (compared Equal) operands)
  | Lt S -> holds (binary (compared LessThanS) operands)
  | Lt U -> holds (binary (compared LessThanU) operands)
  | Gt S -> holds (swapped (compared LessThanS) operands)
  | Gt U -> holds (swapped (compared LessThanU) operands)
  | Le S -> fails (swapped (compared LessThanS) operands)
  | Le U -> fails (swapped (compared LessThanU) operands)
  | Ge S -> fails (binary (compared LessThanS) operands)
  | Ge U -> fails (binary (compared LessThanU) operands)

let number_comparison width (operator : Wasm.Ast.FloatOp.relop) operands =
  let compared comparison = Operator (NumberComparison (width, comparison)) in
  match operator with
  | Eq -> holds (binary (compared Equal) operands)
  | Ne -> fails (binary (compared Equal) operands)
  | Lt -> holds (binary (compared LessThan) operands)
  | Gt -> holds (swapped (compared LessThan) operands)
  | Le -> holds (binary (compared LessThanEqual) operands)
  | Ge -> holds (swapped (compared LessThanEqual) operands)

let integer_unary width (operator : Wasm.Ast.IntOp.unop) =
  match (width, operator) with
  | _, Clz -> Operator (IntegerCount (width, LeadingZeros))
  | _, Ctz -> Operator (IntegerCount (width, TrailingZeros))
  | _, Popcnt -> Operator (IntegerCount (width, CountOnes))
  | _, ExtendS Pack8 -> Operator (Conversion (SignExtend (width, `I8)))
  | _, ExtendS Pack16 -> Operator (Conversion (SignExtend (width, `I16)))
  | `I64, ExtendS Pack32 -> Operator (Conversion (SignExtend (`I64, `I32)))
  | `I32, ExtendS (Pack32 | Pack64) | `I64, ExtendS Pack64 -> invalid_arg "No instruction extends an integer this way."

let number_unary (operator : Wasm.Ast.FloatOp.unop) : NumberUnary.t =
  match operator with
  | Neg -> Negate
  | Abs -> Absolute
  | Ceil -> RoundUp
  | Floor -> RoundDown
  | Trunc -> Truncate
  | Nearest -> Nearest
  | Sqrt -> SquareRoot

let number_binary (operator : Wasm.Ast.FloatOp.binop) : NumberBinary.t =
  match operator with
  | Add -> Add
  | Sub -> Subtract
  | Mul -> Multiply
  | Div -> Divide
  | Min -> Minimum
  | Max -> Maximum
  | CopySign -> CopySign

let integer_conversion target (operator : Wasm.Ast.IntOp.cvtop) operand =
  match operator with
  | WrapI64 -> converted (Narrow `I32) operand
  | ExtendI32 S -> signed64 operand
  | ExtendI32 U -> converted (Widen `I32) operand
  | ReinterpretFloat -> converted (Retype (target :> [ Scalar.integer | Scalar.number ])) operand
  | TruncSatF32 signedness -> converted (saturate target signedness) (converted (Widen `F32) operand)
  | TruncSatF64 signedness -> converted (saturate target signedness) operand
  | TruncF32 _ | TruncF64 _ -> invalid_arg "A trapping truncation is lifted where it traps."

let number_conversion target (operator : Wasm.Ast.FloatOp.cvtop) operand =
  match operator with
  | ConvertI32 S -> converted (ConvertSigned target) (signed64 operand)
  | ConvertI32 U -> converted (ConvertSigned target) (converted (Widen `I32) operand)
  | ConvertI64 S -> converted (ConvertSigned target) operand
  | ConvertI64 U -> converted (ConvertUnsigned target) operand
  | DemoteF64 -> converted (Narrow `F32) operand
  | PromoteF32 -> converted (Widen `F32) operand
  | ReinterpretInt -> converted (Retype (target :> [ Scalar.integer | Scalar.number ])) operand

let numeric (instruction : Wasm.Ast.instr') operands =
  match instruction with
  | Test (I32 Eqz) -> equal `I32 (List.hd operands) (constant `I32 0L)
  | Test (I64 Eqz) -> equal `I64 (List.hd operands) (constant `I64 0L)
  | Compare (I32 operator) -> integer_comparison `I32 operator operands
  | Compare (I64 operator) -> integer_comparison `I64 operator operands
  | Compare (F32 operator) -> number_comparison `F32 operator operands
  | Compare (F64 operator) -> number_comparison `F64 operator operands
  | Unary (I32 operator) -> unary (integer_unary `I32 operator) operands
  | Unary (I64 operator) -> unary (integer_unary `I64 operator) operands
  | Unary (F32 operator) -> unary (Operator (NumberUnary (`F32, number_unary operator))) operands
  | Unary (F64 operator) -> unary (Operator (NumberUnary (`F64, number_unary operator))) operands
  | Binary (I32 operator) -> binary (Operator (IntegerBinary (`I32, integer_binary operator))) operands
  | Binary (I64 operator) -> binary (Operator (IntegerBinary (`I64, integer_binary operator))) operands
  | Binary (F32 operator) -> binary (Operator (NumberBinary (`F32, number_binary operator))) operands
  | Binary (F64 operator) -> binary (Operator (NumberBinary (`F64, number_binary operator))) operands
  | Convert (I32 operator) -> integer_conversion `I32 operator (List.hd operands)
  | Convert (I64 operator) -> integer_conversion `I64 operator (List.hd operands)
  | Convert (F32 operator) -> number_conversion `F32 operator (List.hd operands)
  | Convert (F64 operator) -> number_conversion `F64 operator (List.hd operands)
  | other -> unsupported other
