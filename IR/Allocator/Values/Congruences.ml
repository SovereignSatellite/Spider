type t = int array

let create count = Array.init count Fun.id

let rec find parents value =
  let parent = parents.(value) in
  if parent = value then value
  else
    let root = find parents parent in
    parents.(value) <- root;
    root

let join parents first second =
  let first_root = find parents first in
  parents.(find parents second) <- first_root
