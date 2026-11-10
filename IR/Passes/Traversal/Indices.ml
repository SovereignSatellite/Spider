let exists count predicate =
  let rec from index = index < count && (predicate index || from (index + 1)) in
  from 0

let for_all count predicate = not (exists count (fun index -> not (predicate index)))
