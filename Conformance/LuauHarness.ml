open IR
open Plan
open Running

let header =
  {|local scratch = buffer.create(8)

local function show_i32(value: number): string
    return (string.format("%08x", value))
end

local function show_i64(value: number): string
    buffer.writef64(scratch, 0, value)
    return (string.format("%08x%08x", buffer.readu32(scratch, 4), (buffer.readu32(scratch, 0))))
end

local show_f32 = show_i32
local show_f64 = show_i64

local function show_external(tag: number, value: number?): string
    return (show_i32(if tag == 0 then 0 else value))
end

local function from_halves(low: number, high: number): number
    buffer.writeu32(scratch, 0, low)
    buffer.writeu32(scratch, 4, high)
    return (buffer.readf64(scratch, 0))
end

local function filled_array(length: number, value: any): { any }
    const array: any = table.create(length, value)
    array.length = length
    return array
end

local function run_command(line: number, command: () -> { any }): { any }?
    const ok, results = pcall(command)

    if ok then
        return results
    end

    print(line .. "\ttrap")
    return nil
end

type Import = { space: string, instance: number? }

type Step =
    { kind: "instantiate", line: number, instance: number, imports: { Import } }
    | {
        kind: "invoke",
        line: number,
        instance: number,
        function_: string,
        arguments: { any },
        call: (line: number, closure: { any }, arguments: { any }) -> { string }?,
    }
    | { kind: "get", line: number, instance: number, global: string, read: (box_: { any }) -> string }

local function is_ready(instances: { [number]: any }, imports: { Import }): boolean
    for _, import in imports do
        if import.instance and not instances[import.instance] then
            return false
        end
    end

    return true
end

local function run_steps(spectest: { [string]: any }, steps: { Step })
    const instances = {}

    for _, step in steps do
        if step.kind == "instantiate" then
            if is_ready(instances, step.imports) then
                instances[step.instance] = run_command(step.line, function()
                    const module = require("./module_" .. step.instance)

                    for _, import in step.imports do
                        module.imports[import.space] = if import.instance then instances[import.instance] else spectest
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
            const results = step.call(step.line, instances[step.instance][step.function_], step.arguments)

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
  | I32T | F32T -> Printf.sprintf "0x%Lx" bits
  | I64T | F64T ->
      Printf.sprintf "from_halves(0x%Lx, 0x%Lx)" (Int64.logand bits 0xFFFF_FFFFL) (Int64.shift_right_logical bits 32)

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
        "    memory = { buffer.create(65536), 2 },";
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
    else "const " ^ leaves ^ " = " ^ call ^ "\n        return { " ^ leaves ^ " }"
  in
  let results =
    each_once
      (List.map
         (fun (kind, start) -> shown kind (fun leaf -> "results[" ^ string_of_int (start + leaf + 1) ^ "]"))
         (Kind.positioned signature.results))
  in
  Printf.sprintf
    {|function callers.%s(line: number, closure: { any }, arguments: { any }): { string }?
    const results = run_command(line, function()
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
    {|function readers.%s(box_: { any }): string
    return (%s)
end|}
    name
    (shown kind (fun leaf -> "box_[" ^ string_of_int (leaf + 1) ^ "]"))

let write_module options directory instance =
  let compiled = Plan.compile (TypeArena.create Luau.LuauExtension.extension) options instance in
  let source = Luau.Printer.print (Luau.Build.labelled options (RegionTable.create 0) compiled) in
  Plan.verify compiled;
  Out_channel.with_open_bin
    (Filename.concat directory ("module_" ^ string_of_int instance.number ^ ".luau"))
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
  write_lines (Filename.concat directory "main.luau")
    ([ header; spectest; "local callers = {}" ]
    @ List.map (fun signature -> caller (caller_of signature) signature) signatures
    @ [ "local readers = {}" ]
    @ List.map (fun kind -> reader (reader_of kind) kind) reads
    @ [ "local STEPS: { Step } = {" ]
    @ List.map (fun step -> "    " ^ step ^ ",") table
    @ [ "}"; "run_steps(spectest, STEPS)" ])

let target name options =
  { name; write = write options; run = (fun directory -> Running.run directory "luau" [ "main.luau" ]) }
