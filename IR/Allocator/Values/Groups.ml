let by_key key_count key value items =
  let starts = Array.make (key_count + 1) 0 in
  Array.iter
    (fun item ->
      let group = key item in
      starts.(group) <- starts.(group) + 1)
    items;
  for index = 1 to key_count do
    starts.(index) <- starts.(index) + starts.(index - 1)
  done;
  let order = Array.make (Array.length items) 0 in
  for index = Array.length items - 1 downto 0 do
    let group = key items.(index) in
    starts.(group) <- starts.(group) - 1;
    order.(starts.(group)) <- index
  done;
  (starts, Array.map (fun index -> value items.(index)) order)
