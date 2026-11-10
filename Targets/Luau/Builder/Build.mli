(** The host fills [imports], calls [start], then reads [exports]; [imports[namespace][name]] and [exports[name]] each
    hold a box. *)

open IR
open Tree
open LuauExtension

val whole_runtime : unit -> Statement.t list
val items_within : int -> Passes.Options.t -> (x, string) RegionTable.t -> x CompilationUnit.t -> Statement.t list
val labelled : Passes.Options.t -> (x, string) RegionTable.t -> x CompilationUnit.t -> Statement.t list
