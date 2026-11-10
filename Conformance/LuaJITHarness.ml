open IR
open Plan
open Running

let header =
  {|local ffi = require("ffi")
local scratch = ffi.new("union { int64_t i64; double f64; }")

local function show_i32(value)
    return (bit.tohex(value))
end

local show_i64 = show_i32
local show_f32 = show_i32

local function show_f64(value)
    scratch.f64 = value
    return (bit.tohex(scratch.i64))
end

local function show_external(tag, value)
    return (show_i32(tag == 0 and 0 or value))
end

local function from_bits(bits)
    if bits > 0x7FF0000000000000LL then
        bits = 0x7FF8000000000000LL
    elseif bits < 0LL and bits > 0xFFF0000000000000LL then
        bits = 0xFFF8000000000000LL
    end

    scratch.i64 = bits
    return scratch.f64
end

local bytes = ffi.typeof("struct { int32_t length; uint8_t data[?]; }")

local function new_memory(length)
    local memory = ffi.new(bytes, length)
    memory.length = length
    return memory
end

local function filled_array(length, value)
    local array = {}

    for key = 1, length do
        array[key] = value
    end

    array.length = length
    return array
end

local function run_command(line, command)
    local ok, results = pcall(command)

    if ok then
        return results
    end

    print(line .. "\ttrap")
    return nil
end

local function is_ready(instances, imports)
    for _, import in ipairs(imports) do
        if import.instance and not instances[import.instance] then
            return false
        end
    end

    return true
end

local function run_steps(spectest, steps)
    local instances = {}

    for _, step in ipairs(steps) do
        if step.kind == "instantiate" then
            if is_ready(instances, step.imports) then
                instances[step.instance] = run_command(step.line, function()
                    local module = dofile("module_" .. step.instance .. ".lua")

                    for _, import in ipairs(step.imports) do
                        module.imports[import.space] = import.instance and instances[import.instance] or spectest
                    end

                    module.start()
                    return module.exports
                end)

                if instances[step.instance] then
                    print(step.line .. "\tok\t")
                end
            else
                print(step.line .. "\tuninstantiated")
            end
        elseif not instances[step.instance] then
            print(step.line .. "\tuninstantiated")
        elseif step.kind == "invoke" then
            local results = step.call(step.line, instances[step.instance][step.function_], step.arguments)

            if results then
                print(step.line .. "\tok\t" .. table.concat(results, " "))
            end
        else
            print(step.line .. "\tok\t" .. step.read(instances[step.instance][step.global]))
        end
    end
end
|}

let literal (value_type : Wasm.Types.numtype) bits =
  match value_type with
  | I32T | F32T -> Int64.to_string (Int64.of_int32 (Int64.to_int32 bits))
  | I64T -> Printf.sprintf "0x%LxLL" bits
  | F64T -> Printf.sprintf "from_bits(0x%LxLL)" bits

let each_once values =
  let is_call value =
    String.ends_with ~suffix:")" value && match value.[0] with 'a' .. 'z' | 'A' .. 'Z' -> true | _ -> false
  in
  let last = List.length values - 1 in
  String.concat ", "
    (List.mapi (fun index value -> if index = last && is_call value then "(" ^ value ^ ")" else value) values)

let key name =
  let output = Buffer.create 16 in
  Emission.ByteString.write output name;
  Buffer.contents output

let spectest =
  let print = "{ function() end, {} }" in
  String.concat "\n"
    ([ "local spectest = {" ]
    @ List.map
        (fun name -> "    " ^ name ^ " = " ^ print ^ ",")
        [ "print"; "print_i32"; "print_i64"; "print_f32"; "print_f64"; "print_i32_f32"; "print_f64_f64" ]
    @ [
        "    global_i32 = { " ^ each_once [ literal I32T 666L ] ^ " },";
        "    global_i64 = { " ^ each_once [ literal I64T 666L ] ^ " },";
        "    global_f32 = { "
        ^ each_once [ literal F32T (Int64.logand (Int64.of_int32 (Int32.bits_of_float 666.6)) 0xFFFF_FFFFL) ]
        ^ " },";
        "    global_f64 = { " ^ each_once [ literal F64T (Int64.bits_of_float 666.6) ] ^ " },";
        "    memory = { new_memory(65536), 2 },";
        "    table = { filled_array(10, 0), filled_array(10, nil), filled_array(10, nil), 20 },";
        "}";
      ])

let instantiation instance =
  let imports =
    String.concat ", "
      (List.map
         (fun (space, source) ->
           let from = match source with Spectest -> "nil" | Exported exporter -> string_of_int exporter in
           "{ space = " ^ key space ^ ", instance = " ^ from ^ " }")
         (distinct (List.map (fun import -> (import.module_, import.source)) instance.imports)))
  in
  Printf.sprintf "{ kind = \"instantiate\", line = %d, instance = %d, imports = { %s } }" instance.line instance.number
    imports

let invocation_step caller_of (invocation : invocation) =
  let arguments =
    each_once
      (List.concat_map
         (fun argument ->
           match argument with
           | Number (value_type, bits) -> [ literal value_type bits ]
           | Null -> [ "0"; "nil"; "nil" ]
           | Host 0L -> [ "0"; "nil" ]
           | Host value -> [ "1"; Int64.to_string value ])
         invocation.arguments)
  in
  Printf.sprintf
    "{ kind = \"invoke\", line = %d, instance = %d, function_ = %s, arguments = { %s }, call = callers.%s }"
    invocation.line invocation.instance (key invocation.function_) arguments (caller_of invocation.signature)

let read_step reader_of read =
  Printf.sprintf "{ kind = \"get\", line = %d, instance = %d, global = %s, read = readers.%s }" read.line read.instance
    (key read.global) (reader_of read.kind)

let caller name signature =
  let width = List.fold_left (fun sum kind -> sum + Kind.width kind) 0 in
  let arguments =
    String.concat ", "
      ("closure[2]"
      :: List.init (width signature.parameters) (fun index -> "arguments[" ^ string_of_int (index + 1) ^ "]"))
  in
  let call = "closure[1](" ^ arguments ^ ")" in
  let leaves = String.concat ", " (List.init (width signature.results) (fun leaf -> "result" ^ string_of_int leaf)) in
  let command =
    if leaves = "" then call ^ "\n        return {}"
    else "local " ^ leaves ^ " = " ^ call ^ "\n        return { " ^ leaves ^ " }"
  in
  let results =
    each_once
      (List.map
         (fun (kind, start) -> shown kind (fun leaf -> "results[" ^ string_of_int (start + leaf + 1) ^ "]"))
         (Kind.positioned signature.results))
  in
  Printf.sprintf
    {|function callers.%s(line, closure, arguments)
    local results = run_command(line, function()
        %s
    end)

    if results then
        return { %s }
    end

    return nil
end|}
    name command results

let reader name kind =
  Printf.sprintf
    {|function readers.%s(box_)
    return (%s)
end|}
    name
    (shown kind (fun leaf -> "box_[" ^ string_of_int (leaf + 1) ^ "]"))

let write_module options directory instance =
  let compiled = Plan.compile (TypeArena.create LuaJIT.LuaJITExtension.extension) options instance in
  let source = LuaJIT.Printer.print (LuaJIT.Build.labelled options (RegionTable.create 0) compiled) in
  Plan.verify compiled;
  Out_channel.with_open_bin
    (Filename.concat directory ("module_" ^ string_of_int instance.number ^ ".lua"))
    (fun channel -> Out_channel.output_string channel source)

let write options directory plan =
  let steps = Plan.steps plan in
  List.iter (write_module options directory) (instances steps);
  let signatures, caller_of = callers steps in
  let reads, reader_of = readers steps in
  let table =
    List.map
      (function
        | Instantiate instance | Uninstantiable instance -> instantiation instance
        | Invoke invocation -> invocation_step caller_of invocation
        | Get read -> read_step reader_of read)
      steps
  in
  write_lines (Filename.concat directory "main.lua")
    ([ header; spectest; "local callers = {}" ]
    @ List.map (fun signature -> caller (caller_of signature) signature) signatures
    @ [ "local readers = {}" ]
    @ List.map (fun kind -> reader (reader_of kind) kind) reads
    @ [ "local STEPS = {" ]
    @ List.map (fun step -> "    " ^ step ^ ",") table
    @ [ "}"; "run_steps(spectest, STEPS)" ])

let target name options =
  { name; write = write options; run = (fun directory -> Running.run directory "luajit" [ "main.lua" ]) }
