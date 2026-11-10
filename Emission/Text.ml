let is_upper character = 'A' <= character && character <= 'Z'
let is_lower character = 'a' <= character && character <= 'z'

let write_identifier output case name =
  String.iteri
    (fun index character ->
      let starts_word () =
        (not (is_upper name.[index - 1])) || (index + 1 < String.length name && is_lower name.[index + 1])
      in
      if index > 0 && is_upper character && starts_word () then Buffer.add_char output '_';
      Buffer.add_char output (case character))
    name

let write_name output (name : Name.t) =
  match name with
  | Text text -> Buffer.add_string output text
  | Numbered (prefix, number) ->
      Buffer.add_string output prefix;
      Buffer.add_string output (string_of_int number)
  | Region number ->
      Buffer.add_string output "region_";
      Buffer.add_string output (string_of_int number);
      Buffer.add_char output '_'
  | Register (Fixed number | Overflow number) ->
      Buffer.add_string output "loc_";
      Buffer.add_string output (string_of_int number);
      Buffer.add_char output '_'
  | Helper helper -> write_identifier output Char.lowercase_ascii helper
  | Constant constant -> write_identifier output Char.uppercase_ascii constant

let rec write_items output is_type write items =
  match items with
  | first :: (second :: _ as rest) ->
      write first;
      if not (is_type first && is_type second) then Buffer.add_char output '\n';
      write_items output is_type write rest
  | [ last ] -> write last
  | [] -> ()

let separated output separator write items =
  List.iteri
    (fun index item ->
      if index > 0 then Buffer.add_string output separator;
      write item)
    items

let enclosed output opening closing write items =
  Buffer.add_string output opening;
  separated output ", " write items;
  Buffer.add_string output closing

let write_unsigned output value = Buffer.add_string output (Printf.sprintf "%Lu" value)

let write_source output text =
  Buffer.add_string output text;
  Buffer.add_char output '\n'
