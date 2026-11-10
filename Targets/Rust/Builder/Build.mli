(** The host fills [IMPORTS], calls [run], then reads [EXPORTS]; [IMPORTS[namespace][name]] and [EXPORTS[name]] each
    hold a box. *)

open IR
open Tree
open RustExtension

val whole_runtime : unit -> Item.t list
val labelled : Passes.Options.t -> (x, string) RegionTable.t -> x CompilationUnit.t -> Item.t list
