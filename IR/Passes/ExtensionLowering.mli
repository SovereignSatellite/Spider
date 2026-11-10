open IR
open Definitions

val bounded :
  'x type_arena ->
  Options.t ->
  Options.optimization ->
  'x node ->
  ('x link array -> 'x link) ->
  ('x link array -> 'x link) ->
  (Type.t -> 'x link) ->
  'x link

val allocation :
  'x type_arena ->
  'x node ->
  ('x link -> 'x link -> 'x link) ->
  ('x link -> 'x link) ->
  ('x link array -> 'x link) ->
  'x link

val attempt :
  'x type_arena ->
  'x node ->
  ('x link array -> 'x link array) ->
  ('x link -> 'x link) ->
  ('x link array -> 'x link array) ->
  ('x link array -> 'x link array) ->
  'x link

val run : ('x node -> 'x link option) -> 'x CompilationUnit.t -> unit
