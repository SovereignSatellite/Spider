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

let create types runtime labels =
  let layouts = Layouts.create types in
  let helpers = RuntimeLibrary.NeededHelpers.create runtime in
  { layouts; functions = FunctionQueue.create (); helpers; labels }

let table_name : Name.t = Text "functions"
let function_of context body : Expression.t = Field (Name table_name, FunctionQueue.name context.functions body)
