open IR
open Definitions
open Entities
open Traps

let exception_ context state tag values =
  let contents_type = Context.contents context context.exception_box in
  let contents =
    make
      (SumNew { sum_type = contents_type; variant = Context.thrown context tag })
      (Array.of_list (State.item state (Tag tag) :: values))
  in
  make
    (SumNew { sum_type = context.signatures.exception_; variant = 1 })
    [| make (BoxNew context.exception_box) [| contents |] |]

let catch context state tag reference =
  let parameters = context.signatures.tags.(tag) in
  let result_types = context.tag_box :: parameters in
  let box = result (traps_in context reference 0 [ context.exception_box ] Fun.id) in
  let opened =
    RegionMatch.create context.types
      (result (BoxGet.create context.types box))
      result_types
      (fun variant fields ->
        if variant = Context.thrown context tag then fields else Array.of_list (fields.(0) :: List.map zero parameters))
  in
  let matched = make BoxSame [| Link.output opened 0; State.item state (Tag tag) |] in
  List.init (List.length parameters) (fun index -> Link.output opened (index + 1)) @ [ make I32FromSum [| matched |] ]

let raise state reference =
  let settled = State.settle state in
  ([], { settled with token = result (Throw.create [| reference; settled.token |] [ Type.unit ]) })
