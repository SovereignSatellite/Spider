open IR

type x = [ IR_Memory.t | IR_Rust.t ]

let extension : x Definitions.extension =
  {
    name = (function #IR_Memory.t as node -> IR_Memory.name node | #IR_Rust.t as node -> IR_Rust.name node);
    output_count =
      (fun types node -> function
        | #IR_Memory.t as extended -> IR_Memory.output_count types node extended
        | #IR_Rust.t as extended -> IR_Rust.output_count types node extended);
    output_type =
      (fun types node -> function
        | #IR_Memory.t as extended -> IR_Memory.output_type types node extended
        | #IR_Rust.t as extended -> IR_Rust.output_type types node extended);
    input_type =
      (fun types node -> function
        | #IR_Memory.t as extended -> IR_Memory.input_type types node extended
        | #IR_Rust.t as extended -> IR_Rust.input_type types node extended);
    base = (function #IR_Memory.t as node -> IR_Memory.base node | #IR_Rust.t as node -> IR_Rust.base node);
    holds = (function #IR_Memory.t as node -> IR_Memory.holds node | #IR_Rust.t as node -> IR_Rust.holds node);
    commutative =
      (function #IR_Memory.t as node -> IR_Memory.commutative node | #IR_Rust.t as node -> IR_Rust.commutative node);
    unrepeatable =
      (function #IR_Memory.t as node -> IR_Memory.unrepeatable node | #IR_Rust.t as node -> IR_Rust.unrepeatable node);
    array_access =
      (function #IR_Memory.t as node -> IR_Memory.array_access node | #IR_Rust.t as node -> IR_Rust.array_access node);
    describe = IR_Rust.describe;
  }
