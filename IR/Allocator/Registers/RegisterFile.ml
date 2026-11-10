open IR

type t = { registers : int; class_ : Type.t -> int }

let create registers key =
  let classes = Hashtbl.create 8 in
  {
    registers;
    class_ =
      (fun leaf ->
        let key = key leaf in
        match Hashtbl.find_opt classes key with
        | Some class_ -> class_
        | None ->
            let class_ = Hashtbl.length classes in
            Hashtbl.add classes key class_;
            class_);
  }
