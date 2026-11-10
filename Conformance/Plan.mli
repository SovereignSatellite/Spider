open IR
open Suite

type kind = Number of Wasm.Types.numtype | Reference | External

module Kind : sig
  val width : kind -> int
  val positioned : kind list -> (kind * int) list
end

type source = Spectest | Exported of int
type binding = { module_ : string; name : string; source : source }
type instance = { number : int; line : int; definition : Wasm.Ast.module_; imports : binding list }
type expectation = Returns of value list | Traps | Finishes

module Expectation : sig
  val describe : expectation -> string
end

type argument = Number of Wasm.Types.numtype * int64 | Null | Host of int64
type signature = { parameters : kind list; results : kind list }

type invocation = {
  line : int;
  instance : int;
  function_ : string;
  arguments : argument list;
  signature : signature;
  expectation : expectation;
}

type read = { line : int; instance : int; global : string; kind : kind; expected : value list }
type step = Instantiate of instance | Uninstantiable of instance | Invoke of invocation | Get of read
type disposition = Run of step | Refused of string | Skipped of string | Failed of string
type plan = { test : string; commands : (int * disposition) list }

val steps : plan -> step list
val judged : plan -> (int * expectation) list

(** Lifted afresh each time, since building a target changes the graphs. *)
val compile : ([> IR_Memory.t ] as 'a) TypeArena.t -> Passes.Options.t -> instance -> 'a CompilationUnit.t

val verify : 'x CompilationUnit.t -> unit

(** The host value the harnesses give [ref.extern number], so that null alone is 0. *)
val host : int64 -> int64

val create : test -> plan
