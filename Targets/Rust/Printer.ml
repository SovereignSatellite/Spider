open Emission
open Emission.Text
open Tree

let tuple output write items = enclosed output "(" (if List.length items = 1 then ",)" else ")") write items

let binary_symbol (operator : RustOperator.binary) =
  match operator with
  | Add -> "+"
  | Subtract -> "-"
  | Multiply -> "*"
  | Divide -> "/"
  | Remainder -> "%"
  | BitwiseAnd -> "&"
  | BitwiseOr -> "|"
  | BitwiseExclusiveOr -> "^"
  | Equal -> "=="
  | NotEqual -> "!="
  | LessThan -> "<"
  | LessThanEqual -> "<="
  | And -> "&&"
  | Or -> "||"

let unary_symbol (operator : RustOperator.unary) =
  match operator with Negate -> "-" | Not -> "!" | Dereference -> "*" | Reference -> "&"

let compound (operator : RustOperator.binary) =
  match operator with
  | Add | Subtract | Multiply | Divide | Remainder | BitwiseAnd | BitwiseOr | BitwiseExclusiveOr -> true
  | Equal | NotEqual | LessThan | LessThanEqual | And | Or -> false

let rec write_type output (written : RustType.t) =
  match written with
  | Named name -> write_name output name
  | Generic (name, arguments) -> write_generic output name "<" arguments
  | Function (parameters, result) -> write_function_type output parameters result
  | Pointer target -> write_pointer_type output target
  | Slice element -> write_slice_type output element
  | Tuple elements -> tuple output (write_type output) elements

and write_generic output name opening arguments =
  write_name output name;
  enclosed output opening ">" (write_type output) arguments

and write_function_type output parameters result =
  enclosed output "unsafe fn(" ")" (write_type output) parameters;
  write_result output result

and write_result output result =
  Buffer.add_string output " -> ";
  write_type output result

and write_pointer_type output target =
  Buffer.add_string output "*mut ";
  write_type output target

and write_slice_type output element =
  Buffer.add_char output '[';
  write_type output element;
  Buffer.add_char output ']'

let write_annotation output annotated =
  Buffer.add_string output ": ";
  write_type output annotated

let is_compound (expression : Expression.t) =
  match expression with
  | Binary _ | Unary _ | BooleanToInteger _ | IfElse _ | Cast _ -> true
  | Integer _ | Bytes _ | Name _ | Generic _ | Call _ | Method _ | Field _ | Tuple _ | Array _ | Struct _ | Match _
  | Transmute _ | Closure _ ->
      false

let write_integer output value suffix =
  write_unsigned output value;
  Buffer.add_string output suffix

let write_comment output text =
  Buffer.add_string output "// ";
  ByteString.write_unquoted output text;
  Buffer.add_char output '\n'

let indent output depth =
  for _ = 1 to 4 * depth do
    Buffer.add_char output ' '
  done

let rec write_expression output (written : Expression.t) =
  match written with
  | Integer (value, suffix) -> write_integer output value suffix
  | Bytes bytes ->
      Buffer.add_char output 'b';
      ByteString.write output bytes
  | Name name -> write_name output name
  | Generic (name, arguments) -> write_generic output name "::<" arguments
  | Call (callee, arguments) -> write_call output callee arguments
  | Method (receiver, name, arguments) -> write_method_call output receiver name arguments
  | Field (operand, name) -> write_field_access output operand name
  | Binary (operator, left, right) -> write_binary output operator left right
  | Unary (operator, operand) -> write_unary output operator operand
  | Tuple elements -> tuple output (write_expression output) elements
  | Array elements -> enclosed output "[" "]" (write_expression output) elements
  | Struct (name, fields) -> write_struct output name fields
  | BooleanToInteger boolean -> write_cast output boolean (Named (Text "u32"))
  | IfElse (condition, value, otherwise) -> write_if_else output condition value otherwise
  | Match (tag, values) -> write_match_value output tag values
  | Cast (operand, target) -> write_cast output operand target
  | Transmute (operand, target) -> write_transmute output operand target
  | Closure body -> write_closure output body

and write_parenthesized output expression =
  Buffer.add_char output '(';
  write_expression output expression;
  Buffer.add_char output ')'

and write_operand output expression =
  if is_compound expression then write_parenthesized output expression else write_expression output expression

and write_call output callee arguments =
  (match callee with Field _ -> write_parenthesized output callee | _ -> write_operand output callee);
  enclosed output "(" ")" (write_expression output) arguments

and write_method_call output receiver name arguments =
  write_operand output receiver;
  Buffer.add_char output '.';
  Buffer.add_string output name;
  enclosed output "(" ")" (write_expression output) arguments

and write_field_access output operand name =
  write_operand output operand;
  Buffer.add_char output '.';
  write_name output name

and write_binary output operator left right =
  write_operand output left;
  Buffer.add_char output ' ';
  Buffer.add_string output (binary_symbol operator);
  Buffer.add_char output ' ';
  write_operand output right

and write_unary output operator operand =
  Buffer.add_string output (unary_symbol operator);
  write_operand output operand

and write_struct output name fields =
  write_name output name;
  enclosed output " { " " }"
    (fun (field, value) ->
      write_name output field;
      Buffer.add_string output ": ";
      write_expression output value)
    fields

and write_if_else output condition value otherwise =
  Buffer.add_string output "if ";
  write_expression output condition;
  Buffer.add_string output " { ";
  write_expression output value;
  Buffer.add_string output " } else { ";
  write_expression output otherwise;
  Buffer.add_string output " }"

(** A match ends in [unreachable_unchecked]: its arms cover every variant, and the IR leaves other tags undefined. *)
and write_pattern output variants =
  separated output " | " (fun variant -> Buffer.add_string output (string_of_int variant)) variants

and write_match_value output tag values =
  Buffer.add_string output "match ";
  write_expression output tag;
  Buffer.add_string output " { ";
  List.iter
    (fun (variants, value) ->
      write_pattern output variants;
      Buffer.add_string output " => ";
      write_expression output value;
      Buffer.add_string output ", ")
    values;
  Buffer.add_string output "_ => core::hint::unreachable_unchecked() }"

and write_cast output operand (target : RustType.t) =
  write_operand output operand;
  Buffer.add_string output " as ";
  write_type output target

and write_transmute output operand target =
  Buffer.add_string output "core::mem::transmute::<_, ";
  write_type output target;
  Buffer.add_string output ">(";
  write_expression output operand;
  Buffer.add_char output ')'

and write_closure output body =
  Buffer.add_string output "|| ";
  write_expression output body

let rec write_block output depth statements =
  write_open_block output depth statements;
  Buffer.add_char output '\n'

and write_open_block output depth statements =
  Buffer.add_string output " {\n";
  List.iter (write_statement output (depth + 1)) statements;
  indent output depth;
  Buffer.add_char output '}'

and write_expression_statement output expression =
  write_expression output expression;
  Buffer.add_string output ";\n"

and write_local output name declared value =
  Buffer.add_string output "let mut ";
  write_name output name;
  write_annotation output declared;
  Buffer.add_string output " = ";
  write_expression_statement output value

and write_assign output targets (values : Expression.t list) =
  write_expression output (Expression.grouped targets);
  (match (targets, values) with
  | [ target ], [ Binary (operator, left, right) ] when compound operator && left = target ->
      Buffer.add_char output ' ';
      Buffer.add_string output (binary_symbol operator);
      Buffer.add_string output "= ";
      write_expression output right
  | _ ->
      Buffer.add_string output " = ";
      write_expression output (Expression.grouped values));
  Buffer.add_string output ";\n"

and write_if output depth condition body otherwise =
  Buffer.add_string output "if ";
  write_expression output condition;
  match otherwise with
  | [] -> write_block output depth body
  | _ ->
      write_open_block output depth body;
      Buffer.add_string output " else";
      write_block output depth otherwise

and write_enclosed output depth opening body =
  Buffer.add_string output opening;
  write_block output depth body

and write_return output values =
  write_expression output (Expression.grouped values);
  Buffer.add_char output '\n'

and write_match output depth scrutinee arms =
  Buffer.add_string output "match ";
  write_expression output scrutinee;
  Buffer.add_string output " {\n";
  List.iter
    (fun (variants, body) ->
      indent output (depth + 1);
      write_pattern output variants;
      Buffer.add_string output " =>";
      write_block output (depth + 1) body)
    arms;
  indent output (depth + 1);
  Buffer.add_string output "_ => core::hint::unreachable_unchecked(),\n";
  indent output depth;
  Buffer.add_string output "}\n"

and write_statement output depth (written : Statement.t) =
  indent output depth;
  match written with
  | Local (name, declared, value) -> write_local output name declared value
  | Assign (targets, values) -> write_assign output targets values
  | Call call -> write_expression_statement output call
  | If (condition, body, otherwise) -> write_if output depth condition body otherwise
  | Loop body -> write_enclosed output depth "loop" body
  | Break -> Buffer.add_string output "break;\n"
  | Return values -> write_return output values
  | Match (scrutinee, arms) -> write_match output depth scrutinee arms
  | Comment text -> write_comment output text

let write_parameter output ((name, parameter_type) : Name.t * RustType.t) =
  if name <> Text "_" then Buffer.add_string output "mut ";
  write_name output name;
  write_annotation output parameter_type

let write_function output name parameters result body =
  Buffer.add_string output "pub unsafe fn ";
  write_name output name;
  enclosed output "(" ")" (write_parameter output) parameters;
  write_result output result;
  write_block output 0 body

(** [repr(C)] fixes the field order, so a host's [repr(C)] struct of the same fields in order can read a box. *)
let write_type_definition output name fields =
  Buffer.add_string output "#[derive(Clone, Copy)]\n#[repr(C)]\npub struct ";
  write_name output name;
  Buffer.add_string output " {\n";
  List.iter
    (fun (field, field_type) ->
      indent output 1;
      Buffer.add_string output "pub ";
      write_name output field;
      write_annotation output field_type;
      Buffer.add_string output ",\n")
    fields;
  Buffer.add_string output "}\n"

let write_item output (written : Item.t) =
  match written with
  | Function (name, parameters, result, body) -> write_function output name parameters result body
  | Type (name, fields) -> write_type_definition output name fields
  | Source text -> write_source output text

let print items =
  let output = Buffer.create 65536 in
  Buffer.add_string output
    "#![allow(\n\
    \    unconditional_panic,\n\
    \    unreachable_code,\n\
    \    unsafe_op_in_unsafe_fn,\n\
    \    unused_assignments,\n\
    \    unused_mut,\n\
    \    unused_variables\n\
     )]\n\n";
  let is_type (item : Item.t) = match item with Type _ -> true | _ -> false in
  write_items output is_type (write_item output) items;
  Buffer.contents output
