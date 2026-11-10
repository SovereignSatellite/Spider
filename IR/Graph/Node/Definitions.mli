(** Nodes are compared by identity ([==]). *)
type 'x node = { id : int; kind : 'x kind; mutable inputs : 'x link array }

and 'x link = { node : 'x node; port : int }

and 'x region = {
  region_id : int;
  types : 'x type_arena;
  arguments : 'x node;
  results : 'x node;
  mutable parameter_types : Type.t array;
  mutable result_types : Type.t array;
  mutable region_type : Type.t;
}

and 'x type_arena = {
  mutable definitions : TypeDefinition.t array; [@atomic]
  mutable count : int;
  interned : Type.t StructuralType.Table.t;
  extended : (TypeDefinition.extension, Type.t) Hashtbl.t;
  lock : Mutex.t;
  extension : 'x extension;
}

and 'x extension = {
  name : 'x -> string;
  output_count : 'x type_arena -> 'x node -> 'x -> int;
  output_type : 'x type_arena -> 'x node -> 'x -> int -> Type.t;
  input_type : 'x type_arena -> 'x node -> 'x -> int -> Type.t;
  base : 'x -> base;
  holds : 'x -> int;
  commutative : 'x -> bool;
  unrepeatable : 'x -> bool;
  array_access : 'x -> array_access option;
  describe : TypeDefinition.extension -> string;
}

and base = Operation | UnaryOperation | BinaryOperation | ViewOperation | Write

(** Accesses [width] elements of the array (input 0) from the index (input 1) plus [offset], least significant first; a
    write writes its input 2. *)
and array_access = { offset : int; width : int }

(** The output's part at [output_path] is, unchanged, the part at [input_path] of the reader's input. The reader itself
    uses that value only where [reads] is set, as a write uses the reference it writes. *)
and 'x flow = {
  reader : 'x node;
  input : int;
  input_path : Path.t;
  output : 'x link;
  output_path : Path.t;
  reads : bool;
}

and 'x kind =
  | RegionArguments of 'x region
  | RegionResults of 'x region
  | RegionReference of 'x region
  | FunctionApply
  | FunctionTry of Type.t list
      (** Applies the callee as [FunctionApply] does, giving variant 1 holding its results, or variant 0 holding the
          payload of a [Throw] that ended the application, undefined unless the payload has the payload types. *)
  | FunctionRetype of Type.t  (** Applying a function through a type other than its own is undefined. *)
  | RegionMatch of 'x region array
  | RegionRepeat of 'x region
  | I8Constant of int
  | I16Constant of int
  | I32Constant of int
  | I64Constant of int64
  | F32Constant of int32
  | F64Constant of float
  | Operator of Operator.t
  | Throw of Type.t Dynarray.t
      (** Once its inputs, the payload, are available, ends every application up to the innermost running [FunctionTry],
          or the program if none runs; it never produces its outputs, so they can be any region's results. *)
  | SumNew of { sum_type : Type.t; variant : int }
  | SumFromI32 of { sum_type : Type.t; variants : int list; tagged : bool }
      (** [variants] are those whose fields have the values' types; a tag naming any other is undefined. *)
  | I32FromSum
  | ArrayNew of { array_type : Type.t; allocated : Type.t }
  | ArrayOf of { element_type : Type.t; bytes : string; array_type : Type.t }
      (** [bytes] holds each element in turn, least significant byte first. *)
  | ArraySize
  | ArrayGet of Type.t  (** An index out of bounds is undefined. *)
  | ArraySet of Type.t  (** An index out of bounds is undefined. *)
  | ArrayFill of Type.t  (** A range out of bounds is undefined. *)
  | ArrayCopy of Type.t  (** Copies as if through a temporary; a range out of bounds is undefined. *)
  | ArraySame
  | ArrayFinalize of Type.t
      (** Ends the array's lifetime; using it afterwards for anything but reaching a program boundary is undefined. *)
  | BoxNew of Type.t
  | BoxGet of Type.t
  | BoxSet of Type.t
  | BoxRetype of Type.t  (** Accessing a box through a type other than the one it was created with is undefined. *)
  | BoxSame
  | BoxFinalize of Type.t
      (** Ends the box's lifetime; using it afterwards for anything but reaching a program boundary is undefined. *)
  | Import of { import_type : Type.t; namespace : string; identifier : string }
  | Export of string
  | Fence  (** The value, available only once it and every waited link are. *)
  | Extension of 'x
      (** An extension's kind holds only data, never a node or region, so passes compare kinds structurally. *)
