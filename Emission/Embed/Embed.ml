let () =
  let output = Buffer.create 65536 in
  let quoted text =
    Buffer.add_char output '"';
    Buffer.add_string output (String.escaped text);
    Buffer.add_char output '"'
  in
  Buffer.add_string output "let resources = [\n";
  Array.iter
    (fun path ->
      Buffer.add_string output "  (";
      quoted path;
      Buffer.add_string output ", ";
      quoted (In_channel.with_open_bin path In_channel.input_all);
      Buffer.add_string output ");\n")
    (Array.sub Sys.argv 1 (Array.length Sys.argv - 1));
  Buffer.add_string output "]\n";
  Buffer.output_buffer stdout output
