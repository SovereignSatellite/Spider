open IR
open Definitions
open KnownValue

let optimization =
  Options.optimization "redirect-invariant-values" Graph Boundaries 2
    "Read a Match or loop output known to equal an outer value as that value."

let redirected output knowledge =
  match knowledge with
  | SameAs link when Constant.is link.node -> Constant.copy link
  | SameAs link -> link
  | _ -> output

let redirect_match types matcher =
  Array.init (Node.output_count types matcher) (fun port ->
      redirected (Link.output matcher port) (across_branches types matcher port (fun link -> SameAs link)))

let exit_value types repeat port =
  match field_through types Fun.id 0 port (RegionRepeat.step repeat) with
  | SameAs link -> ( match carried_outside types Fun.id repeat link with Some link -> SameAs link | None -> Unknown)
  | _ -> Unknown

let redirect_repeat types repeat =
  Array.init (Node.output_count types repeat) (fun port ->
      redirected (Link.output repeat port) (exit_value types repeat port))

let redirect types node =
  match node.kind with
  | RegionMatch _ -> Some (redirect_match types node)
  | RegionRepeat _ -> Some (redirect_repeat types node)
  | _ -> None

let run (table : 'x Readers.table) = Rewrite.nodes (redirect table.region.types) table.nodes
