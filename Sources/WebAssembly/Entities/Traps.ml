open IR
open Entities

let payload_types context = [ context.signatures.exception_; Type.unit ]

(** A trap throws null, which no clause catches, as [throw_ref] of null traps; being able to trap orders nothing, so its
    token is a constant. *)
let trapped context = Array.of_list (List.map zero (payload_types context))

let traps_in context sum trapping result_types compute =
  RegionMatch.create context.types sum result_types (fun variant fields ->
      if variant = trapping then
        let throw = Throw.create (trapped context) result_types in
        Array.init (List.length result_types) (Link.output throw)
      else compute fields)

let allocate context length initial =
  ArrayNew.or_throw context.types (trapped context) (ArrayNew.create context.types length initial)
