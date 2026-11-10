open IR
open Allocator
open Emission
open Tree
open LuaJITExtension

type t = {
  layouts : x Layouts.t;
  functions : x FunctionQueue.t;
  helpers : RuntimeLibrary.NeededHelpers.t;
  labels : (x, string) RegionTable.t;
}

val create : x TypeArena.t -> RuntimeLibrary.t -> (x, string) RegionTable.t -> t
val table_name : Name.t
val function_of : t -> x Region.t -> Expression.t
