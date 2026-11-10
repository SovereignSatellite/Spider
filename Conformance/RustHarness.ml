open IR
open Plan
open Running

let header =
  {|use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;

type Exports = BTreeMap<Vec<u8>, usize>;
type Imports = BTreeMap<Vec<u8>, Exports>;

/// A box of the value, whose struct has the value's layout.
fn boxed<T>(value: T) -> usize {
    Box::into_raw(Box::new(value)) as usize
}

fn show_i32(value: u32) -> String {
    format!("{value:08x}")
}

fn show_i64(value: u64) -> String {
    format!("{value:016x}")
}

fn show_f32(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

fn show_f64(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn show_external(tag: u32, value: *mut ()) -> String {
    show_i32(if tag == 0 { 0 } else { value as usize as u32 })
}

fn run_command<T>(line: u32, command: impl FnOnce() -> T) -> Option<T> {
    match catch_unwind(AssertUnwindSafe(command)) {
        Ok(value) => Some(value),
        Err(_) => {
            println!("{line}\ttrap");
            None
        }
    }
}

#[repr(C)]
struct SpectestMemory {
    bytes: *mut [u8],
    maximum: u32,
}

#[repr(C)]
struct SpectestTable {
    tags: *mut [u32],
    functions: *mut [unsafe fn()],
    environments: *mut [*mut ()],
    maximum: u32,
}

unsafe fn spectest_print(_: *mut ()) {}
unsafe fn spectest_print_i32(_: *mut (), _: u32) {}
unsafe fn spectest_print_i64(_: *mut (), _: u64) {}
unsafe fn spectest_print_f32(_: *mut (), _: f32) {}
unsafe fn spectest_print_f64(_: *mut (), _: f64) {}
unsafe fn spectest_print_i32_f32(_: *mut (), _: u32, _: f32) {}
unsafe fn spectest_print_f64_f64(_: *mut (), _: f64, _: f64) {}

unsafe fn unreachable() {
    unsafe { core::hint::unreachable_unchecked() }
}

#[repr(C)]
struct Reference {
    tag: u32,
    function: unsafe fn(),
    environment: *mut (),
}

#[repr(C)]
struct External {
    tag: u32,
    value: *mut (),
}

#[repr(C)]
struct Closure<F> {
    function: F,
    environment: *mut (),
}

struct Import {
    space: &'static [u8],
    name: &'static [u8],
    instance: Option<usize>,
}

#[derive(Clone, Copy)]
struct Module {
    imports: &'static Mutex<Imports>,
    exports: &'static Mutex<Exports>,
    run: unsafe fn(),
}

enum Step {
    Instantiate {
        line: u32,
        instance: usize,
        imports: &'static [Import],
        module: Module,
    },
    Invoke {
        line: u32,
        instance: usize,
        function: &'static [u8],
        arguments: &'static [u64],
        call: unsafe fn(u32, usize, &[u64]) -> Option<Vec<String>>,
    },
    Get {
        line: u32,
        instance: usize,
        global: &'static [u8],
        read: unsafe fn(usize) -> String,
    },
}

fn run_steps() {
    unsafe {
        let spectest = spectest();
        let mut instances: Vec<Option<Exports>> = (0..INSTANCES).map(|_| None).collect();

        for step in STEPS {
            match *step {
                Step::Instantiate { line, instance, imports, module } => {
                    instances[instance] = if imports.iter().all(|import| import.instance.is_none_or(|from| instances[from].is_some())) {
                        let mut linked = module.imports.lock().unwrap();

                        for import in imports {
                            let from = match import.instance {
                                Some(from) => instances[from].as_ref().unwrap(),
                                None => &spectest,
                            };

                            let values = linked.entry(import.space.to_vec()).or_default();
                            values.insert(import.name.to_vec(), from[import.name]);
                        }

                        drop(linked);
                        let instantiated = run_command(line, || (module.run)());

                        if instantiated.is_some() {
                            println!("{line}\tok\t");
                        }

                        instantiated.map(|_| core::mem::take(&mut *module.exports.lock().unwrap()))
                    } else {
                        println!("{line}\tuninstantiated");
                        None
                    };
                }
                Step::Invoke { line, instance, function, arguments, call } => match &instances[instance] {
                    Some(exports) => {
                        if let Some(shown) = call(line, exports[function], arguments) {
                            println!("{line}\tok\t{}", shown.join(" "));
                        }
                    }
                    None => println!("{line}\tuninstantiated"),
                },
                Step::Get { line, instance, global, read } => match &instances[instance] {
                    Some(exports) => println!("{line}\tok\t{}", read(exports[global])),
                    None => println!("{line}\tuninstantiated"),
                },
            }
        }
    }
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    std::thread::Builder::new().stack_size(1 << 30).spawn(run_steps).unwrap().join().unwrap();
}
|}

let rust_type (value_type : Wasm.Types.numtype) =
  match value_type with I32T -> "u32" | I64T -> "u64" | F32T -> "f32" | F64T -> "f64"

let literal (value_type : Wasm.Types.numtype) bits =
  match value_type with
  | F32T -> Printf.sprintf "f32::from_bits(0x%Lxu32)" bits
  | F64T -> Printf.sprintf "f64::from_bits(0x%Lxu64)" bits
  | I32T | I64T -> Printf.sprintf "0x%Lx%s" bits (rust_type value_type)

let key name =
  let output = Buffer.create 16 in
  Buffer.add_char output 'b';
  Emission.ByteString.write output name;
  Buffer.contents output

let leaves (kind : kind) =
  match kind with
  | Number number_type -> (rust_type number_type, [ ("", rust_type number_type) ])
  | Reference -> ("Reference", [ (".tag", "u32"); (".function", "unsafe fn()"); (".environment", "*mut ()") ])
  | External -> ("External", [ (".tag", "u32"); (".value", "*mut ()") ])

let leaf_types kind = List.map snd (snd (leaves kind))

let spectest =
  let print name (parameters : Wasm.Types.numtype list) =
    let placeholders = String.concat ", " (List.init (List.length parameters + 1) (fun _ -> "_")) in
    ( name,
      "boxed(Closure { function: spectest_" ^ name ^ " as unsafe fn(" ^ placeholders
      ^ "), environment: core::ptr::null_mut::<()>() })" )
  in
  [
    "unsafe fn spectest() -> Exports {";
    "    let memory = boxed(SpectestMemory {";
    "        bytes: Box::into_raw(vec![0u8; 65536].into_boxed_slice()),";
    "        maximum: 2,";
    "    });";
    "";
    "    let table = boxed(SpectestTable {";
    "        tags: Box::into_raw(vec![0u32; 10].into_boxed_slice()),";
    "        functions: Box::into_raw(vec![unreachable as unsafe fn(); 10].into_boxed_slice()),";
    "        environments: Box::into_raw(vec![core::ptr::null_mut::<()>(); 10].into_boxed_slice()),";
    "        maximum: 20,";
    "    });";
    "";
    "    BTreeMap::from([";
  ]
  @ List.map
      (fun (name, box_) -> "        (" ^ key name ^ ".to_vec(), " ^ box_ ^ "),")
      [
        print "print" [];
        print "print_i32" [ I32T ];
        print "print_i64" [ I64T ];
        print "print_f32" [ F32T ];
        print "print_f64" [ F64T ];
        print "print_i32_f32" [ I32T; F32T ];
        print "print_f64_f64" [ F64T; F64T ];
        ("global_i32", "boxed(" ^ literal I32T 666L ^ ")");
        ("global_i64", "boxed(" ^ literal I64T 666L ^ ")");
        ( "global_f32",
          "boxed(" ^ literal F32T (Int64.logand (Int64.of_int32 (Int32.bits_of_float 666.6)) 0xFFFF_FFFFL) ^ ")" );
        ("global_f64", "boxed(" ^ literal F64T (Int64.bits_of_float 666.6) ^ ")");
        ("memory", "memory");
        ("table", "table");
      ]
  @ [ "    ])"; "}" ]

let passed_number index (value_type : Wasm.Types.numtype) =
  let bits = "arguments[" ^ string_of_int index ^ "]" in
  match value_type with
  | I32T -> bits ^ " as u32"
  | I64T -> bits
  | F32T -> "f32::from_bits(" ^ bits ^ " as u32)"
  | F64T -> "f64::from_bits(" ^ bits ^ ")"

let passed arguments =
  List.concat
    (snd
       (List.fold_left_map
          (fun index (kind : kind) ->
            match kind with
            | Number value_type -> (index + 1, [ passed_number index value_type ])
            | Reference -> (index, [ "0"; "unreachable"; "core::ptr::null_mut()" ])
            | External ->
                let value = "arguments[" ^ string_of_int index ^ "]" in
                (index + 1, [ "(" ^ value ^ " != 0) as u32"; value ^ " as usize as *mut ()" ]))
          0 arguments))

let caller name signature =
  let parameters = "*mut ()" :: List.concat_map leaf_types signature.parameters in
  let results = List.concat_map leaf_types signature.results in
  let result_type =
    match results with [ single ] -> "(" ^ single ^ ",)" | _ -> "(" ^ String.concat ", " results ^ ")"
  in
  let passed = passed signature.parameters in
  let arguments = String.concat ", " ("environment" :: passed) in
  let shown_results =
    String.concat ", "
      (List.map
         (fun (kind, start) -> shown kind (fun leaf -> "results." ^ string_of_int (start + leaf)))
         (Kind.positioned signature.results))
  in
  let function_type = "unsafe fn(" ^ String.concat ", " parameters ^ ") -> " ^ result_type in
  Printf.sprintf
    {|unsafe fn %s(line: u32, closure: usize, arguments: &[u64]) -> Option<Vec<String>> {
    unsafe {
        let Closure { function, environment } = (closure as *const Closure<%s>).read();
        run_command(line, || function(%s)).map(|results| vec![%s])
    }
}|}
    name function_type arguments shown_results

let reader name kind =
  let structure, fields = leaves kind in
  let leaf index = "(*(box_ as *const " ^ structure ^ "))" ^ fst (List.nth fields index) in
  Printf.sprintf {|unsafe fn %s(box_: usize) -> String {
    unsafe { %s }
}|} name (shown kind leaf)

let instantiation instance =
  let imports =
    String.concat ", "
      (List.map
         (fun import ->
           let from =
             match import.source with Spectest -> "None" | Exported exporter -> "Some(" ^ string_of_int exporter ^ ")"
           in
           "Import { space: " ^ key import.module_ ^ ", name: " ^ key import.name ^ ", instance: " ^ from ^ " }")
         instance.imports)
  in
  let name = "module_" ^ string_of_int instance.number in
  Printf.sprintf
    "Step::Instantiate { line: %d, instance: %d, imports: &[%s], module: Module { imports: &%s::IMPORTS, exports: \
     &%s::EXPORTS, run: %s::run } }"
    instance.line instance.number imports name name name

let invocation_step caller_of (invocation : invocation) =
  let arguments =
    String.concat ", "
      (List.filter_map
         (fun argument ->
           match argument with Number (_, bits) | Host bits -> Some (Printf.sprintf "0x%Lx" bits) | Null -> None)
         invocation.arguments)
  in
  Printf.sprintf "Step::Invoke { line: %d, instance: %d, function: %s, arguments: &[%s], call: %s }" invocation.line
    invocation.instance (key invocation.function_) arguments (caller_of invocation.signature)

let read_step reader_of read =
  Printf.sprintf "Step::Get { line: %d, instance: %d, global: %s, read: %s }" read.line read.instance (key read.global)
    (reader_of read.kind)

let write_module options directory instance =
  let compiled = Plan.compile (TypeArena.create Rust.RustExtension.extension) options instance in
  let source = Rust.Printer.print (Rust.Build.labelled options (RegionTable.create 0) compiled) in
  Plan.verify compiled;
  Out_channel.with_open_bin
    (Filename.concat directory ("module_" ^ string_of_int instance.number ^ ".rs"))
    (fun channel -> Out_channel.output_string channel source)

let write options directory plan =
  let steps = Plan.steps plan in
  let instances = instances steps in
  List.iter (write_module options directory) instances;
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
  write_lines (Filename.concat directory "main.rs")
    (List.map (fun instance -> "mod module_" ^ string_of_int instance.number ^ ";") instances
    @ [ header ] @ spectest
    @ List.map (fun signature -> caller (caller_of signature) signature) signatures
    @ List.map (fun kind -> reader (reader_of kind) kind) reads
    @ [ "const INSTANCES: usize = " ^ string_of_int (List.length instances) ^ ";"; "static STEPS: &[Step] = &[" ]
    @ List.map (fun step -> "    " ^ step ^ ",") table
    @ [ "];" ])

let compile_and_run directory =
  match Running.run directory "rustc" [ "--edition"; "2024"; "-A"; "warnings"; "main.rs"; "-o"; "main" ] with
  | Finished _ ->
      let program = Filename.concat directory "main" in
      let run = Running.run directory program [] in
      Sys.remove program;
      run
  | Stopped (_, reason) -> Stopped ("", reason)

let target name options = { name; write = write options; run = compile_and_run }
