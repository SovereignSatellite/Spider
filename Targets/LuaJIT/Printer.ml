open Emission
open Emission.Text
open Tree

let binary_symbol (operator : LuaJITOperator.binary) =
  match operator with
  | Add -> "+"
  | Subtract -> "-"
  | Multiply -> "*"
  | Divide -> "/"
  | Modulo -> "%"
  | Equal -> "=="
  | NotEqual -> "~="
  | LessThan -> "<"
  | LessThanEqual -> "<="
  | GreaterThanEqual -> ">="
  | And -> "and"
  | Or -> "or"

let unary_symbol (operator : LuaJITOperator.unary) = match operator with Negate -> "-" | Not -> "not "

(** A negative literal is an operand only in parentheses: a [-] before it would start a comment, and [^] binds tighter
    than its sign. *)
let is_compound (expression : Expression.t) =
  match expression with
  | Binary _ | Unary _ | Function _ | Table _ | Record _ -> true
  | Number bits -> Float.sign_bit (Int64.float_of_bits bits)
  (* LuaJIT: LuaJIT is untyped, its integers print signed, and the FFI has long literals. *)
  | Integer value -> value < 0L
  | Nil | Long _ | Bytes _ | Name _ | Call _ | Field _ | Index _ -> false

let rec starts_with_parenthesis (expression : Expression.t) =
  match expression with
  | Call (operand, _) | Field (operand, _) | Index (operand, _) ->
      is_compound operand || starts_with_parenthesis operand
  (* LuaJIT: LuaJIT is untyped, its integers print signed, and the FFI has long literals. *)
  | Nil | Integer _ | Number _ | Long _ | Bytes _ | Name _ | Binary _ | Unary _ | Table _ | Record _ | Function _ ->
      false

(* LuaJIT: LuaJIT is untyped, its integers print signed, and the FFI has long literals. *)
let write_integer output value = Buffer.add_string output (Int64.to_string value)

(** In hexadecimal, as LuaJIT reads a literal of 2^63 or more as the [int64_t] holding its bits. *)
let write_long output value = Buffer.add_string output (Printf.sprintf "0x%LxLL" value)

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
  (* LuaJIT: LuaJIT is untyped, its integers print signed, and the FFI has long literals. *)
  | Integer value -> write_integer output value
  | Number bits -> Decimal.write_number output bits
  | Long value -> write_long output value
  | Bytes bytes -> ByteString.write output bytes
  | Name name -> write_name output name
  | Call (callee, arguments) -> write_call output depth callee arguments
  | Field (operand, name) -> write_field_access output depth operand name
  | Index (operand, key) -> write_index output depth operand key
  | Binary (operator, left, right) -> write_binary output depth operator left right
  | Unary (operator, operand) -> write_unary output depth operator operand
  (* LuaJIT: LuaJIT is untyped, its integers print signed, and the FFI has long literals. *)
  | Table ([], []) -> Buffer.add_string output "{}"
  | Table (values, fields) -> write_table output depth values fields
  | Record fields -> write_record output depth fields
  | Function (parameters, body) -> write_function output depth parameters body

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

and write_function output depth parameters body =
  (* LuaJIT: LuaJIT is untyped, its integers print signed, and the FFI has long literals. *)
  enclosed output "function(" ")" (write_name output) parameters;
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

(** LuaJIT rejects a statement starting with a parenthesis on the line after another unless a [;] ends that one, and it
    has no empty statement, so no [;] may come before a block's first statement. *)
and start_statement output follows first = if follows && starts_with_parenthesis first then Buffer.add_char output ';'

(** A lone call stays bare, as it may assign several targets. *)
and write_assigned_values output depth values =
  match values with [ value ] -> write_expression output depth value | _ -> write_each_once output depth values

(* LuaJIT: LuaJIT is untyped, its integers print signed, and the FFI has long literals. *)
and write_local output depth name value =
  Buffer.add_string output "local ";
  write_name output name;
  Option.iter
    (fun value ->
      Buffer.add_string output " = ";
      write_expression output depth value)
    value;
  Buffer.add_char output '\n'

and write_assign output depth follows targets values =
  start_statement output follows (List.hd targets);
  separated output ", " (write_expression output depth) targets;
  Buffer.add_string output " = ";
  write_assigned_values output depth values;
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

and write_statement output depth follows written =
  indent output depth;
  match written with
  (* LuaJIT: LuaJIT is untyped, its integers print signed, and the FFI has long literals. *)
  | Local (name, value) -> write_local output depth name value
  | Assign (targets, values) -> write_assign output depth follows targets values
  | Call call -> write_call_statement output depth follows call
  | If (condition, body, otherwise) -> write_if output depth condition body otherwise
  | Do body -> write_enclosed output depth "do" body
  | Loop body -> write_enclosed output depth "while true do" body
  | Break -> Buffer.add_string output "break\n"
  | Return values -> write_return output depth values
  | Source text -> write_source output text
  | Comment text -> write_comment output text

let print statements =
  let output = Buffer.create 65536 in
  (* LuaJIT: LuaJIT is untyped, its integers print signed, and the FFI has long literals. *)
  write_items output (fun _ -> false) (write_statement output 0 false) statements;
  Buffer.contents output
