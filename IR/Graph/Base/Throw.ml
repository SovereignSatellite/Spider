open Definitions

let create inputs result_types = Node.create (Throw (Dynarray.of_list result_types)) inputs

let producing throw value_type =
  match throw.kind with
  | Throw result_types -> (
      match Dynarray.find_index (fun described -> described = value_type) result_types with
      | Some port -> Link.output throw port
      | None ->
          Dynarray.add_last result_types value_type;
          Link.output throw (Dynarray.length result_types - 1))
  | _ -> invalid_arg "Expected a Throw."
