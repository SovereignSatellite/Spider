open IR
open Definitions
open Passes
open RustValues

(** Rust's [as] from an integer depends only on its value, and to an integer type only on the value modulo 2^width. *)
let composes types source middle target =
  match (integer_type types source, integer_type types middle, integer_type types target) with
  | Some _, Some _, _ when keeps types source middle -> true
  | Some _, Some (middle_bits, _), Some (target_bits, _) -> target_bits <= middle_bits
  | _ -> false

let composed types node =
  let inner = node.inputs.(0) in
  let source = inner.node.inputs.(0) in
  let target = Node.output_type types node 0 in
  match composes types (Link.type_ types source) (Link.type_ types inner) target with
  | true when Link.type_ types source = target -> Some source
  | true -> Some (Numbers.cast target source)
  | false -> None

let casts = Options.optimization "compose-rust-casts" Integer Identities 1 "Compose nested Rust casts into one."
let sums = Options.optimization "simplify-rust-sums" Integer Identities 1 "Remove Rust's wrapping_add of zero."

let summed types node =
  match (integer types node.inputs.(0), integer types node.inputs.(1)) with
  | _, Some 0L -> Some node.inputs.(0)
  | Some 0L, _ -> Some node.inputs.(1)
  | _ -> None

let simplified options types node =
  match node.kind with
  | Extension (`RustCast _) -> (
      match node.inputs.(0).node.kind with
      | Extension (`RustCast _) -> Options.applied options casts (fun () -> composed types node)
      | _ -> None)
  | Extension `RustWrappingAdd -> Options.applied options sums (fun () -> summed types node)
  | _ -> None

let fold options types node = Option.map (fun simplified -> [| simplified |]) (simplified options types node)
