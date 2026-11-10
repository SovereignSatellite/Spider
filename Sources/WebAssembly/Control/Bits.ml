type t = { mutable words : int array }

let width = 32
let create () = { words = Array.make 8 0 }

let contains bits item =
  let word = item lsr 5 in
  word < Array.length bits.words && bits.words.(word) land (1 lsl (item land 31)) <> 0

let add bits item =
  let word = item lsr 5 in
  if word >= Array.length bits.words then begin
    let grown = Array.make (max (word + 1) (Array.length bits.words * 2)) 0 in
    Array.blit bits.words 0 grown 0 (Array.length bits.words);
    bits.words <- grown
  end;
  let bit = 1 lsl (item land 31) in
  let absent = bits.words.(word) land bit = 0 in
  bits.words.(word) <- bits.words.(word) lor bit;
  absent

let remove bits item =
  let word = item lsr 5 in
  if word < Array.length bits.words then bits.words.(word) <- bits.words.(word) land lnot (1 lsl (item land 31))

let add_all bits items = Array.iter (fun item -> ignore (add bits item)) items
let clear bits = Array.fill bits.words 0 (Array.length bits.words) 0
let copy bits = { words = Array.copy bits.words }

let to_array bits =
  let items = Dynarray.create () in
  Array.iteri
    (fun index word ->
      for bit = 0 to width - 1 do
        if word land (1 lsl bit) <> 0 then Dynarray.add_last items ((index * width) + bit)
      done)
    bits.words;
  Dynarray.to_array items

let of_list items =
  let bits = create () in
  List.iter (fun item -> ignore (add bits item)) items;
  bits
