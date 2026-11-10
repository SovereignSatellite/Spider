(** This repository holds the specification tests as wast2json output. *)

val artifacts : string
val download : unit -> string

type value =
  | Bits of Wasm.Types.numtype * int64
  | CanonicalNaN of Wasm.Types.numtype
  | ArithmeticNaN of Wasm.Types.numtype
  | Reference of bool
  | External of int64
  | Unsupported of string

module Value : sig
  val describe : value -> string
end

type action_kind = Invoke | Get
type action = { kind : action_kind; module_ : string option; field : string; arguments : value list }

type command =
  | Module of { binary : string option; name : string option }
  | ModuleInstance of { definition : string; instance : string option }
  | ModuleDefinition of { binary : string; name : string option }
  | Register of { name : string option; registered : string }
  | AssertUninstantiable of string
  | AssertReturn of action * value list
  | AssertTrap of action * string
  | Action of action
  | Unplanned of string

type test = { name : string; commands : (int * command) list }

val tests : string -> test list
