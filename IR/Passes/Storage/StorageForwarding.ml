open IR
open Definitions
open Accesses

let optimization =
  Options.optimization "forward-storage" Storage Forwarding 2 "Forward a stored value to the reads that see it."

let shifted index apart =
  match Offsets.split index with
  | None, offset ->
      Link.output (Node.create (I32Constant (Int64.to_int (Int64.add offset apart) land 0xFFFF_FFFF)) [||]) 0
  | Some origin, offset -> Offsets.offset `I32 origin (Int64.logand (Int64.add offset apart) low_word)

(** The reference a Match or Repeat output gives back as the one the node was given, through writes and the region nodes
    within it. *)
let rec given types reference =
  match reference.node.kind with
  | RegionMatch _ -> (
      match
        KnownValue.across_branches types reference.node reference.port (fun link -> KnownValue.SameAs (seen types link))
      with
      | SameAs outer -> Some outer
      | _ -> None)
  | RegionRepeat _ -> (
      match KnownValue.field_through types (seen types) 0 reference.port (RegionRepeat.step reference.node) with
      | SameAs link -> KnownValue.carried_outside types (seen types) reference.node link
      | _ -> None)
  | _ -> None

and seen types reference =
  let rooted = unwritten types reference in
  match given types rooted with Some outer -> seen types outer | None -> rooted

type 'x found = Value of 'x link * 'x link | Earliest of 'x link
type 'x walk = { passed : 'x NodeSet.t; accesses : 'x access Dynarray.t }

let read_literal access bytes reference first element_width width =
  let start = Int64.to_int (Int64.mul first element_width) in
  let bits = ref 0L in
  for position = start + Int64.to_int width - 1 downto start do
    bits := Int64.logor (Int64.shift_left !bits 8) (Int64.of_int (Char.code bytes.[position]))
  done;
  Value (Constant.of_bits access.value_type !bits, reference)

let rec find_at_write types walk access reference node written value =
  match same_index written access with
  | Some true -> Value (value, reference)
  | Some false -> pass types walk access node written.reference
  | None -> Earliest reference

and find_at_bulk_write types walk access reference node start count =
  match overlap access start count with
  | Inside apart -> inside types walk access reference apart
  | Outside -> pass types walk access node node.inputs.(0)
  | Unknown -> Earliest reference

and find types walk access reference =
  let node = reference.node in
  match (write types node, bulk_write node) with
  | Some (written, value), _ -> find_at_write types walk access reference node written value
  | _, Some (start, count) -> find_at_bulk_write types walk access reference node start count
  | _ -> created types walk access reference

and pass types walk access passed reference =
  ignore (NodeSet.add walk.passed passed);
  find types walk access reference

and find_through_copy types walk access reference node apart =
  let copied = { access with reference = node.inputs.(2); index = shifted node.inputs.(3) apart; offset = 0L } in
  Dynarray.add_last walk.accesses copied;
  match pass types walk copied node copied.reference with Value _ as found -> found | Earliest _ -> Earliest reference

and inside types walk access reference apart =
  let node = reference.node in
  match node.kind with
  | ArrayCopy _ -> find_through_copy types walk access reference node apart
  | _ -> filled access node.inputs.(3) reference

and filled access value reference =
  if access.width = 1L then Value (value, reference)
  else
    match Constant.bits value with
    | Some repeated -> Value (Constant.of_bits access.value_type (Int64.mul repeated 0x0101_0101_0101_0101L), reference)
    | None -> Earliest reference

and created types walk access reference =
  match creation reference with
  | Some ({ kind = BoxNew _; _ } as box) -> Value (box.inputs.(0), reference)
  | Some ({ kind = ArrayNew _; _ } as allocation) -> filled access allocation.inputs.(1) reference
  | Some { kind = ArrayOf { element_type; bytes; _ }; _ } -> held access element_type bytes reference
  | _ -> crossed types walk access reference

and crossed types walk access reference =
  match given types reference with
  | Some outer when not (changes types access reference.node) -> pass types walk access reference.node outer
  | _ -> Earliest reference

and held access element_type bytes reference =
  let element_width = Int64.of_int (Type.bits element_type / 8) in
  let width = Int64.mul access.width element_width in
  match Option.map (Int64.add access.offset) (Constant.bits access.index) with
  | Some first
    when Int64.unsigned_compare (Int64.add (Int64.mul first element_width) width) (Int64.of_int (String.length bytes))
         <= 0 ->
      read_literal access bytes reference first element_width width
  | _ -> Earliest reference

(** Only writers the read waits for and the origin does not can change what it sees: the others may run after the read
    or ran before the origin. *)
let changed types writers walk after origin =
  List.exists
    (fun writer ->
      (not (NodeSet.mem walk.passed writer))
      && Dynarray.exists (fun access -> changes types access writer) walk.accesses)
    (Writers.besides writers after origin.node)

let reread read reference =
  let inputs = Node.inputs read in
  inputs.(0) <- reference;
  Link.output (Copying.node read inputs) 0

let forwarding types writers outside read access =
  let access = { access with reference = outside access.reference; index = outside access.index } in
  let after = List.map outside (Array.to_list read.inputs) in
  let walk = { passed = NodeSet.create 16; accesses = Dynarray.of_list [ access ] } in
  match find types walk access access.reference with
  | Value (value, origin) when not (changed types writers walk after origin) -> Some (value, Fun.id)
  | Earliest earliest
    when (not (Link.equal earliest access.reference)) && not (changed types writers walk after earliest) ->
      Some (earliest, reread read)
  | _ -> None

let forward_read types writers node =
  Option.map
    (fun (outer, within) -> [| within outer |])
    (Option.bind (read types node) (forwarding types writers Fun.id node))

let created_size types node =
  match creation (root types node.inputs.(0)) with
  | Some ({ kind = ArrayNew _; _ } as allocation) -> Some [| allocation.inputs.(0) |]
  | Some { kind = ArrayOf { element_type; bytes; _ }; _ } ->
      Some [| Link.output (Node.create (I32Constant (String.length bytes / (Type.bits element_type / 8))) [||]) 0 |]
  | _ -> None

let known_size types node = match node.kind with ArraySize -> created_size types node | _ -> None

let forward types writers node =
  match forward_read types writers node with Some _ as forwarded -> forwarded | None -> known_size types node

let reads_elsewhere types access reader =
  match read types reader with Some other -> same_index other access = Some false | None -> false

let rec overwritten_by_write types readers access next later =
  match same_index later access with
  | Some true -> true
  | Some false -> overwritten types readers access (Link.output next 0)
  | None -> false

and overwritten_by_next types readers access next =
  match write types next with Some (later, _) -> overwritten_by_write types readers access next later | None -> false

and overwritten types readers access reference =
  match List.filter (fun (reader, _) -> not (reads_elsewhere types access reader)) (readers reference) with
  | [ (next, 0) ] -> overwritten_by_next types readers access next
  | _ -> false

let writes_back types access value =
  match read types value.node with
  | Some earlier -> Link.equal earlier.reference access.reference && same_index earlier access = Some true
  | None -> false

let writes_nothing node = match bulk_write node with Some (_, count) -> Constant.bits count = Some 0L | None -> false

let reduce types readers node =
  let unobserved =
    writes_nothing node
    ||
    match write types node with
    | Some (access, value) -> overwritten types readers access (Link.output node 0) || writes_back types access value
    | None -> false
  in
  if unobserved then Some [| node.inputs.(0) |] else None

(** A nested read of what outer links carry sees what the same read outside would. *)
let forward_into types writers routes (nested : 'x Nested.t) =
  let outside link = if Constant.is link.node then link else LinkTable.find nested.outer link in
  let carried link = Constant.is link.node || LinkTable.mem nested.outer link in
  Array.iter
    (fun node ->
      match read types node with
      | Some access when Array.for_all carried node.inputs ->
          Option.iter
            (fun (outer, within) ->
              Dynarray.add_last routes { Branch.path = nested.path; link = Link.output node 0; outer; within })
            (forwarding types writers outside node access)
      | _ -> ())
    nested.nodes

let run (table : 'x Readers.table) =
  let types = table.region.types in
  let forwarded = Rewrite.nodes (forward types (Writers.create types)) table.nodes in
  let table = if forwarded then Readers.renewed table else table in
  let readers = Readers.of_link table table.region in
  let reduced = Rewrite.nodes (reduce types readers) table.nodes in
  let table = if reduced then Readers.renewed table else table in
  let routes = Dynarray.create () in
  Nested.visit table.nodes (forward_into types (Writers.create types) routes);
  Branch.route types table (Dynarray.to_list routes);
  forwarded || reduced
