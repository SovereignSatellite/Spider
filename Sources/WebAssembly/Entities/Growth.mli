open IR
open Definitions
open Entities

val grow :
  'x context ->
  'x State.t ->
  Fields.key ->
  ('x link -> 'x link) ->
  'x link ->
  'x link ->
  ('x link -> 'x link) ->
  'x link list * 'x State.t
