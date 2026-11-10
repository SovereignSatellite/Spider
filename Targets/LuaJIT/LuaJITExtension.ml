open IR

type x = [ IR_Memory.t | IR_LuaJIT.t ]

let extension : x Definitions.extension =
  {
    name = (function #IR_Memory.t as node -> IR_Memory.name node | #IR_LuaJIT.t as node -> IR_LuaJIT.name node);
    output_count =
      (fun types node -> function
        | #IR_Memory.t as extended -> IR_Memory.output_count types node extended
        | #IR_LuaJIT.t as extended -> IR_LuaJIT.output_count types node extended);
    output_type =
      (fun types node -> function
        | #IR_Memory.t as extended -> IR_Memory.output_type types node extended
        | #IR_LuaJIT.t as extended -> IR_LuaJIT.output_type types node extended);
    input_type =
      (fun types node -> function
        | #IR_Memory.t as extended -> IR_Memory.input_type types node extended
        | #IR_LuaJIT.t as extended -> IR_LuaJIT.input_type types node extended);
    base = (function #IR_Memory.t as node -> IR_Memory.base node | #IR_LuaJIT.t as node -> IR_LuaJIT.base node);
    holds = (function #IR_Memory.t as node -> IR_Memory.holds node | #IR_LuaJIT.t as node -> IR_LuaJIT.holds node);
    commutative =
      (function
      | #IR_Memory.t as node -> IR_Memory.commutative node
      | #IR_LuaJIT.t as node -> IR_LuaJIT.commutative node);
    unrepeatable =
      (function
      | #IR_Memory.t as node -> IR_Memory.unrepeatable node
      | #IR_LuaJIT.t as node -> IR_LuaJIT.unrepeatable node);
    array_access =
      (function
      | #IR_Memory.t as node -> IR_Memory.array_access node
      | #IR_LuaJIT.t as node -> IR_LuaJIT.array_access node);
    describe = IR_LuaJIT.describe;
  }
