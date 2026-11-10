open Definitions

type 'x t = 'x region

let next = Atomic.make 0
let function_type types parameter_types result_types = TypeArena.intern types (Function (parameter_types, result_types))
let parameter region index = Link.output region.arguments index
let parameters region = Array.init (Array.length region.parameter_types) (parameter region)
let parameter_types region = Array.to_list region.parameter_types
let result_types region = Array.to_list region.result_types
let result_count region = Array.length region.result_types
let result_type region index = region.result_types.(index)
let define region links = region.results.inputs <- links

let create ?build types parameter_types result_types =
  let region_type = function_type types parameter_types result_types in
  let region_id = Atomic.fetch_and_add next 1 in
  let arguments_id = Atomic.fetch_and_add Node.next 1 in
  let results_id = Atomic.fetch_and_add Node.next 1 in
  let rec region =
    {
      region_id;
      types;
      arguments = { id = arguments_id; kind = RegionArguments region; inputs = [||] };
      results = { id = results_id; kind = RegionResults region; inputs = [||] };
      parameter_types = Array.of_list parameter_types;
      result_types = Array.of_list result_types;
      region_type;
    }
  in
  Option.iter (fun build -> define region (build region)) build;
  region

let redefine region parameter_types result_types build =
  region.result_types <- Array.of_list result_types;
  region.region_type <- function_type region.types parameter_types result_types;
  region.parameter_types <- Array.of_list parameter_types;
  define region (build region)
