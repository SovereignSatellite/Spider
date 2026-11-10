open IR

type x = [ IR_Memory.t | IR_Luau.t ]

let extension : x Definitions.extension =
  {
    name = (function #IR_Memory.t as node -> IR_Memory.name node | #IR_Luau.t as node -> IR_Luau.name node);
    output_count =
      (fun types node -> function
        | #IR_Memory.t as extended -> IR_Memory.output_count types node extended
        | #IR_Luau.t as extended -> IR_Luau.output_count types node extended);
    output_type =
      (fun types node -> function
        | #IR_Memory.t as extended -> IR_Memory.output_type types node extended
        | #IR_Luau.t as extended -> IR_Luau.output_type types node extended);
    input_type =
      (fun types node -> function
        | #IR_Memory.t as extended -> IR_Memory.input_type types node extended
        | #IR_Luau.t as extended -> IR_Luau.input_type types node extended);
    base = (function #IR_Memory.t as node -> IR_Memory.base node | #IR_Luau.t as node -> IR_Luau.base node);
    holds = (function #IR_Memory.t as node -> IR_Memory.holds node | #IR_Luau.t as node -> IR_Luau.holds node);
    commutative =
      (function #IR_Memory.t as node -> IR_Memory.commutative node | #IR_Luau.t as node -> IR_Luau.commutative node);
    unrepeatable =
      (function #IR_Memory.t as node -> IR_Memory.unrepeatable node | #IR_Luau.t as node -> IR_Luau.unrepeatable node);
    array_access =
      (function #IR_Memory.t as node -> IR_Memory.array_access node | #IR_Luau.t as node -> IR_Luau.array_access node);
    describe = IR_Luau.describe;
  }
