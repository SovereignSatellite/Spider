open IR

let in_parallel work functions =
  let bodies = Array.of_list (List.rev functions) in
  let taken = Atomic.make 0 in
  let failure = Atomic.make None in
  let run added () =
    try
      let rec take () =
        let index = Atomic.fetch_and_add taken 1 in
        if index < Array.length bodies then begin
          work added bodies.(index);
          take ()
        end
      in
      take ()
    with error ->
      Atomic.set failure (Some (error, Printexc.get_raw_backtrace ()));
      Atomic.set taken (Array.length bodies)
  in
  let helpers =
    if Domain.is_main_domain () then Int.max 0 (min (Domain.recommended_domain_count ()) (Array.length bodies) - 1)
    else 0
  in
  let workers =
    Array.init helpers (fun _ ->
        let added = RegionSet.create 16 in
        (Domain.spawn (run added), added))
  in
  let joined = RegionSet.create 16 in
  run joined ();
  Array.iter
    (fun (domain, added) ->
      Domain.join domain;
      RegionSet.union joined added)
    workers;
  Option.iter (fun (error, backtrace) -> Printexc.raise_with_backtrace error backtrace) (Atomic.get failure);
  joined
