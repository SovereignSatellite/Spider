let hex = "0123456789abcdef"

let write_unquoted output bytes =
  String.iter
    (fun character ->
      let value = Char.code character in
      if value >= 0x20 && value < 0x7F && character <> '"' && character <> '\\' then Buffer.add_char output character
      else begin
        Buffer.add_string output "\\x";
        Buffer.add_char output hex.[value lsr 4];
        Buffer.add_char output hex.[value land 15]
      end)
    bytes

let write output bytes =
  Buffer.add_char output '"';
  write_unquoted output bytes;
  Buffer.add_char output '"'
