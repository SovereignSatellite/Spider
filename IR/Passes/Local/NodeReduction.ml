open IR
open Definitions

let canonical_offsets =
  Options.optimization "canonicalize-offsets" Integer Identities 1 "Give each constant offset one shape, x + (a - b)."

let offset_comparisons =
  Options.optimization "compare-through-offsets" Integer Comparisons 1
    "Compare an offset's origin with the constant less the offset."

let fences =
  Options.optimization "reduce-fences" Graph Ordering 1 "Wait for a fence's inputs where only its reader reads it."

let throws =
  Options.optimization "reduce-throws" Graph Ordering 1
    "Replace an operation reading what a throw gives with the throw."

let reduce_offset node =
  Option.bind (Offsets.offset_of node) (fun (width, outer) ->
      match Offsets.split node.inputs.(0) with
      | Some origin, inner when not (Link.equal origin node.inputs.(0)) ->
          Some
            [|
              Offsets.offset width origin (Int64.logand (Int64.add inner outer) (Constant.mask (Type.of_scalar width)));
            |]
      | _ -> None)

let reduce_equality width node =
  let left = node.inputs.(0) in
  match (Offsets.split left, Constant.bits node.inputs.(1)) with
  | (Some origin, by), Some bound when not (Link.equal origin left) ->
      Some
        [|
          Link.output
            (Node.create
               (Operator (IntegerComparison (width, Equal)))
               [| origin; Constant.of_bits (Type.of_scalar width) (Int64.sub bound by) |])
            0;
        |]
  | _ -> None

let is_fence node = match node.kind with Fence -> true | _ -> false

(** Waiting for a fence only the reader reads is waiting for its inputs. *)
let absorbs readers reader link =
  is_fence link.node
  && Readers.is_read readers readers.region link
  && Readers.only_by readers readers.region reader link.node

(** Only fences wait for the read, so it orders nothing. *)
let unobserved types readers node =
  Option.is_some (Accesses.read types node)
  && Readers.is_read readers readers.region (Link.output node 0)
  && List.for_all
       (fun (reader, input) -> is_fence reader && input > 0)
       (Readers.of_link readers readers.region (Link.output node 0))

let rec chain readers fence =
  let value = fence.inputs.(0) in
  if absorbs readers fence value then
    let fences, origin = chain readers value.node in
    (fence :: fences, origin)
  else ([ fence ], value)

(** Waiting for a constant, an operation whose operands alone give its result, an unobserved read or an absorbed fence
    is waiting for its inputs. *)
let waited types readers fences value =
  let seen = LinkTable.create 16 in
  LinkTable.replace seen value ();
  let found = Dynarray.create () in
  let rec wait reader link =
    let node = link.node in
    if not (LinkTable.mem seen link) then begin
      LinkTable.replace seen link ();
      if
        Constant.is node || Node.unary types node || Node.binary types node || unobserved types readers node
        || absorbs readers reader link
      then Array.iter (wait node) node.inputs
      else Dynarray.add_last found link
    end
  in
  List.iter (fun fence -> Array.iter (wait fence) (Array.sub fence.inputs 1 (Array.length fence.inputs - 1))) fences;
  Dynarray.to_array found

let reduce_fence types readers node =
  let fences, value = chain readers node in
  let waited = waited types readers fences value in
  let inputs = Array.append [| value |] waited in
  if Array.length waited = 0 then Some [| value |]
  else if Array.equal Link.equal inputs node.inputs then None
  else Some [| Link.output (Node.create Fence inputs) 0 |]

(** A node reading what a throw never produces never runs, so the throw's outputs stand for its own. *)
let thrown types node throw =
  Array.init (Node.output_count types node) (fun port -> Throw.producing throw (Node.output_type types node port))

let reduce_operation options types readers node =
  let applied = Options.applied options in
  match node.kind with
  | Fence -> applied fences (fun () -> reduce_fence types readers node)
  | Operator (IntegerComparison (width, Equal)) -> applied offset_comparisons (fun () -> reduce_equality width node)
  | _ -> applied canonical_offsets (fun () -> reduce_offset node)

let reduce options types readers node =
  match Array.find_opt (fun input -> match input.node.kind with Throw _ -> true | _ -> false) node.inputs with
  | Some input when options throws -> Some (thrown types node input.node)
  | _ -> reduce_operation options types readers node
