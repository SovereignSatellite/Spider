(** The mantissa and power of ten of the shortest decimal that reads back as the positive value, the nearest of them
    where several do. The first precision that reads back ends in no zero, as the one before would have read back. *)
let shortest value =
  let decimal text =
    match String.split_on_char 'e' text with
    | [ digits; exponent ] ->
        (Int64.of_string (String.concat "" (String.split_on_char '.' digits)), int_of_string exponent)
    | _ -> invalid_arg ("Not a decimal in exponent form: " ^ text)
  in
  let reads_back exponent mantissa = Float.equal (float_of_string (Printf.sprintf "%Lde%d" mantissa exponent)) value in
  let rec at precision =
    let mantissa, first = decimal (Printf.sprintf "%.*e" (precision - 1) value) in
    let exponent = first - (precision - 1) in
    match List.find_opt (reads_back exponent) [ mantissa; Int64.pred mantissa; Int64.succ mantissa ] with
    | Some found -> (found, exponent)
    | None -> at (precision + 1)
  in
  at 1

let write_number output bits =
  let value = Int64.float_of_bits bits in
  if Float.sign_bit value then Buffer.add_char output '-';
  if Float.abs value = 0.0 then Buffer.add_char output '0'
  else begin
    let mantissa, exponent = shortest (Float.abs value) in
    let digits = Int64.to_string mantissa in
    let count = String.length digits in
    let scale = exponent + count in
    if scale > 17 || scale < -3 then begin
      Buffer.add_char output digits.[0];
      if count > 1 then begin
        Buffer.add_char output '.';
        Buffer.add_substring output digits 1 (count - 1)
      end;
      Buffer.add_string output (if scale - 1 < 0 then "E-" else "E+");
      if abs (scale - 1) < 10 then Buffer.add_char output '0';
      Buffer.add_string output (string_of_int (abs (scale - 1)))
    end
    else if scale > 0 then begin
      for index = 0 to scale - 1 do
        Buffer.add_char output (if index < count then digits.[index] else '0')
      done;
      if count > scale then begin
        Buffer.add_char output '.';
        Buffer.add_substring output digits scale (count - scale)
      end
    end
    else begin
      Buffer.add_string output "0.";
      for _ = 1 to -scale do
        Buffer.add_char output '0'
      done;
      Buffer.add_string output digits
    end
  end
