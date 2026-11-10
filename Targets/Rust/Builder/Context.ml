open IR
open Allocator
open Emission
open Tree
open RustExtension

type t = {
  layouts : x Layouts.t;
  rust : RustTypes.t;
  functions : x FunctionQueue.t;
  helpers : RuntimeLibrary.NeededHelpers.t;
  labels : (x, string) RegionTable.t;
}

let create types runtime labels =
  let layouts = Layouts.create types in
  let helpers = RuntimeLibrary.NeededHelpers.create runtime in
  { layouts; rust = RustTypes.create types layouts helpers; functions = FunctionQueue.create (); helpers; labels }

let function_of context body : Expression.t = Name (FunctionQueue.name context.functions body)
