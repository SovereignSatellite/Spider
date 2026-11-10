open IR
open Definitions
open Passes
open RustValues

let fold_cast types node =
  let target = Node.output_type types node 0 in
  match integer_type types target with
  | Some (_, false) -> Option.map (Constant.of_bits target) (integer types (Link.output node 0))
  | _ -> None

let fold_wrapping_add types node =
  let target = Node.output_type types node 0 in
  match (integer_type types target, integer types node.inputs.(0), integer types node.inputs.(1)) with
  | Some (_, false), Some left, Some right -> Some (Constant.of_bits target (Int64.add left right))
  | _ -> None

let casts = Options.optimization "fold-rust-casts" Integer Folding 1 "Fold Rust's as on constants."
let sums = Options.optimization "fold-rust-sums" Integer Folding 1 "Fold Rust's wrapping_add on constants."

let folded options types node =
  match node.kind with
  | Extension (`RustCast _) -> Options.applied options casts (fun () -> fold_cast types node)
  | Extension `RustWrappingAdd -> Options.applied options sums (fun () -> fold_wrapping_add types node)
  | _ -> None

let fold options types node = Option.map (fun folded -> [| folded |]) (folded options types node)
