open Emission
open Emission.Text
open Tree

(* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
let write_empty_table output = Buffer.add_string output "{}"

let binary_symbol (operator : LuauOperator.binary) =
  match operator with
  | Add -> "+"
  | Subtract -> "-"
  | Multiply -> "*"
  (* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
  | Divide -> "/"
  | FloorDivide -> "//"
  | Modulo -> "%"
  | Equal -> "=="
  | NotEqual -> "~="
  | LessThan -> "<"
  | LessThanEqual -> "<="
  | GreaterThanEqual -> ">="
  | And -> "and"
  | Or -> "or"

let unary_symbol (operator : LuauOperator.unary) = match operator with Negate -> "-" | Not -> "not "

(* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
let compound (operator : LuauOperator.binary) =
  match operator with
  | Add | Subtract | Multiply | Divide | FloorDivide | Modulo -> true
  | Equal | NotEqual | LessThan | LessThanEqual | GreaterThanEqual | And | Or -> false

let rec write_type output (written : LuauType.t) =
  match written with
  | Named name -> write_name output name
  | Generic (name, arguments) -> write_generic_type output name arguments
  | Function (parameters, results) -> write_function_type output parameters results
  | Union members -> separated output " | " (write_union_member output) members
  | Table [] -> write_empty_table output
  | Table fields -> enclosed output "{ " " }" (write_field output) fields
  | Sequence element -> enclosed output "{ " " }" (write_type output) [ element ]

and write_parenthesized_types output types = enclosed output "(" ")" (write_type output) types

and write_generic_type output name arguments =
  write_name output name;
  enclosed output "<" ">" (write_type output) arguments

and write_function_type output parameters results =
  write_parenthesized_types output parameters;
  Buffer.add_string output " -> ";
  write_results output results

and write_union_member output member_type =
  match member_type with
  | Function _ -> write_parenthesized_types output [ member_type ]
  | _ -> write_type output member_type

and write_field output (name, field_type) =
  write_name output name;
  write_annotation output field_type

and write_annotation output annotated =
  Buffer.add_string output ": ";
  write_type output annotated

and write_results output results =
  match results with
  | [ Function _ ] | [ Union _ ] -> write_parenthesized_types output results
  | [ single ] -> write_type output single
  | _ -> write_parenthesized_types output results

let write_signature_results output results =
  match results with
  | [] -> ()
  | [ single ] -> write_annotation output single
  | _ ->
      Buffer.add_string output ": ";
      write_parenthesized_types output results

(** A negative literal is an operand only in parentheses: a [-] before it would start a comment, and [^] binds tighter
    than its sign. *)
let is_compound (expression : Expression.t) =
  match expression with
  (* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
  | Binary _ | Unary _ | IfElse _ | Cast _ | Function _ | Table _ | Record _ -> true
  | Number bits -> Float.sign_bit (Int64.float_of_bits bits)
  | Nil | Integer _ | Bytes _ | Name _ | Call _ | Field _ | Index _ -> false

let rec starts_with_parenthesis (expression : Expression.t) =
  match expression with
  | Call (operand, _) | Field (operand, _) | Index (operand, _) ->
      is_compound operand || starts_with_parenthesis operand
  (* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
  | Nil | Integer _ | Number _ | Bytes _ | Name _ | Binary _ | Unary _ | IfElse _ | Cast _ | Table _ | Record _
  | Function _ ->
      false

let write_comment output text =
  Buffer.add_string output "-- ";
  ByteString.write_unquoted output text;
  Buffer.add_char output '\n'

let indent output depth =
  for _ = 1 to depth do
    Buffer.add_char output '\t'
  done

let rec write_expression output depth (written : Expression.t) =
  match written with
  | Nil -> Buffer.add_string output "nil"
  (* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
  | Integer value -> write_unsigned output value
  | Number bits -> Decimal.write_number output bits
  | Bytes bytes -> ByteString.write output bytes
  | Name name -> write_name output name
  | Call (callee, arguments) -> write_call output depth callee arguments
  | Field (operand, name) -> write_field_access output depth operand name
  | Index (operand, key) -> write_index output depth operand key
  | Binary (operator, left, right) -> write_binary output depth operator left right
  | Unary (operator, operand) -> write_unary output depth operator operand
  (* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
  | IfElse (condition, value, otherwise) -> write_if_else output depth condition value otherwise
  | Cast (operand, target) -> write_cast output depth operand target
  | Table ([], []) -> write_empty_table output
  | Table (values, fields) -> write_table output depth values fields
  | Record fields -> write_record output depth fields
  (* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
  | Function (native, parameters, results, body) -> write_function output depth native parameters results body

and write_parenthesized output depth expression =
  Buffer.add_char output '(';
  write_expression output depth expression;
  Buffer.add_char output ')'

and write_operand output depth expression =
  if is_compound expression then write_parenthesized output depth expression
  else write_expression output depth expression

and write_each_once output depth values =
  let last = List.length values - 1 in
  List.iteri
    (fun index (value : Expression.t) ->
      if index > 0 then Buffer.add_string output ", ";
      match value with
      | Call _ when index = last -> write_parenthesized output depth value
      | _ -> write_expression output depth value)
    values

and write_call output depth callee arguments =
  write_operand output depth callee;
  Buffer.add_char output '(';
  write_each_once output depth arguments;
  Buffer.add_char output ')'

and write_field_access output depth operand name =
  write_operand output depth operand;
  Buffer.add_char output '.';
  write_name output name

and write_index output depth operand key =
  write_operand output depth operand;
  Buffer.add_char output '[';
  write_expression output depth key;
  Buffer.add_char output ']'

and write_binary output depth operator left right =
  write_operand output depth left;
  Buffer.add_char output ' ';
  Buffer.add_string output (binary_symbol operator);
  Buffer.add_char output ' ';
  write_operand output depth right

and write_unary output depth operator operand =
  Buffer.add_string output (unary_symbol operator);
  write_operand output depth operand

(* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
and write_if_else output depth condition value otherwise =
  Buffer.add_string output "if ";
  write_expression output depth condition;
  Buffer.add_string output " then ";
  write_expression output depth value;
  Buffer.add_string output " else ";
  write_expression output depth otherwise

and write_cast output depth operand target =
  write_operand output depth operand;
  Buffer.add_string output " :: ";
  write_type output target

and write_table output depth values fields =
  Buffer.add_string output "{ ";
  write_each_once output depth values;
  List.iteri
    (fun index (name, value) ->
      if index > 0 || values <> [] then Buffer.add_string output ", ";
      write_name output name;
      Buffer.add_string output " = ";
      write_expression output depth value)
    fields;
  Buffer.add_string output " }"

and write_record output depth fields =
  Buffer.add_string output "{\n";
  List.iter
    (fun (name, value) ->
      indent output (depth + 1);
      write_name output name;
      Buffer.add_string output " = ";
      write_expression output (depth + 1) value;
      Buffer.add_string output ",\n")
    fields;
  indent output depth;
  Buffer.add_char output '}'

(* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
and write_function output depth native parameters results body =
  if native then Buffer.add_string output "@native ";
  enclosed output "function(" ")" (write_field output) parameters;
  write_signature_results output results;
  Buffer.add_char output '\n';
  write_block output (depth + 1) body;
  indent output depth;
  Buffer.add_string output "end"

and write_block output depth statements =
  let follows previous (statement : Statement.t) = previous || match statement with Comment _ -> false | _ -> true in
  ignore
    (List.fold_left
       (fun follow statement ->
         write_statement output depth follow statement;
         follows follow statement)
       false statements)

and write_line output depth text =
  indent output depth;
  Buffer.add_string output text;
  Buffer.add_char output '\n'

(** Luau rejects a statement starting with a parenthesis on the line after another unless a [;] ends that one, and it
    has no empty statement, so no [;] may come before a block's first statement. *)
and start_statement output follows first = if follows && starts_with_parenthesis first then Buffer.add_char output ';'

(** A lone call stays bare, as it may assign several targets. *)
and write_assigned_values output depth values =
  match values with [ value ] -> write_expression output depth value | _ -> write_each_once output depth values

(* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
and write_binding output depth keyword name declared value =
  Buffer.add_string output keyword;
  write_name output name;
  Option.iter (write_annotation output) declared;
  Option.iter
    (fun value ->
      Buffer.add_string output " = ";
      write_expression output depth value)
    value;
  Buffer.add_char output '\n'

(* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
and write_assign output depth follows targets (values : Expression.t list) =
  start_statement output follows (List.hd targets);
  separated output ", " (write_expression output depth) targets;
  (* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
  (match (targets, values) with
  | [ target ], [ Binary (operator, left, right) ] when compound operator && left = target ->
      Buffer.add_char output ' ';
      Buffer.add_string output (binary_symbol operator);
      Buffer.add_string output "= ";
      write_expression output depth right
  | _ ->
      Buffer.add_string output " = ";
      write_assigned_values output depth values);
  Buffer.add_char output '\n'

and write_call_statement output depth follows call =
  start_statement output follows call;
  write_expression output depth call;
  Buffer.add_char output '\n'

and write_if output depth condition body otherwise =
  Buffer.add_string output "if ";
  write_expression output depth condition;
  Buffer.add_string output " then\n";
  write_block output (depth + 1) body;
  if otherwise <> [] then begin
    write_line output depth "else";
    write_block output (depth + 1) otherwise
  end;
  write_line output depth "end"

and write_enclosed output depth opening body =
  Buffer.add_string output opening;
  Buffer.add_char output '\n';
  write_block output (depth + 1) body;
  write_line output depth "end"

and write_return output depth values =
  Buffer.add_string output "return ";
  write_each_once output depth values;
  Buffer.add_char output '\n'

(* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
and write_type_definition output name definition =
  Buffer.add_string output "export type ";
  write_name output name;
  Buffer.add_string output " = ";
  write_type output definition;
  Buffer.add_char output '\n'

and write_statement output depth follows written =
  indent output depth;
  match written with
  (* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
  | Local (name, declared, value) -> write_binding output depth "local " name (Some declared) value
  | Const (name, declared, value) -> write_binding output depth "const " name declared (Some value)
  | Assign (targets, values) -> write_assign output depth follows targets values
  | Call call -> write_call_statement output depth follows call
  | If (condition, body, otherwise) -> write_if output depth condition body otherwise
  | Do body -> write_enclosed output depth "do" body
  | Loop body -> write_enclosed output depth "while true do" body
  | Break -> Buffer.add_string output "break\n"
  | Return values -> write_return output depth values
  (* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
  | Type (name, definition) -> write_type_definition output name definition
  | Source text -> write_source output text
  | Comment text -> write_comment output text

let print statements =
  let output = Buffer.create 65536 in
  (* Luau: Luau has types, constants, floor division, compound assignment and if-expressions, and its integers print unsigned. *)
  Buffer.add_string output "--!strict\n--!nolint LocalUnused\n--!nolint UnreachableCode\n\n";
  let is_type (statement : Statement.t) = match statement with Type _ -> true | _ -> false in
  write_items output is_type (write_statement output 0 false) statements;
  Buffer.contents output
