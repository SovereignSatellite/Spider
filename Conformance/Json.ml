type t = Number of int | String of string | List of t list | Object of (string * t) list

let parse text =
  let position = ref 0 in
  let peek () = text.[!position] in
  let advance () = incr position in
  let rec value () =
    match peek () with
    | '{' ->
        advance ();
        Object (sequence '}' member)
    | '[' ->
        advance ();
        List (sequence ']' value)
    | '"' -> String (string ())
    | _ ->
        let start = !position in
        while not (String.contains ",]}" (peek ())) do
          advance ()
        done;
        Number (int_of_string (String.sub text start (!position - start)))
  and sequence : 'element. char -> (unit -> 'element) -> 'element list =
   fun closing element ->
    if peek () = closing then begin
      advance ();
      []
    end
    else
      let first = element () in
      let ended = peek () = closing in
      advance ();
      first :: (if ended then [] else sequence closing element)
  and member () =
    let key = string () in
    advance ();
    (key, value ())
  and string () =
    advance ();
    let output = Buffer.create 16 in
    let control () =
      let code = int_of_string ("0x" ^ String.sub text !position 4) in
      position := !position + 4;
      Char.chr code
    in
    let rec characters () =
      let character = peek () in
      advance ();
      match character with
      | '"' -> ()
      | '\\' ->
          let escaped = peek () in
          advance ();
          (match escaped with
          | 'n' -> Buffer.add_char output '\n'
          | 't' -> Buffer.add_char output '\t'
          | 'r' -> Buffer.add_char output '\r'
          | 'b' -> Buffer.add_char output '\b'
          | 'f' -> Buffer.add_char output '\012'
          | 'u' -> Buffer.add_char output (control ())
          | other -> Buffer.add_char output other);
          characters ()
      | other ->
          Buffer.add_char output other;
          characters ()
    in
    characters ();
    Buffer.contents output
  in
  value ()

let members element = match element with Object members -> members | _ -> invalid_arg "Expected an object."
let property element name = List.assoc_opt name (members element)
let get element name = List.assoc name (members element)
let text element = match element with String text -> text | _ -> failwith "Expected a string."
