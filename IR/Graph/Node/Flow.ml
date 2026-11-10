open Definitions

type 'x t = 'x flow

let whole reader input output = { reader; input; input_path = Whole; output; output_path = Whole; reads = false }
let written writer = { (whole writer 0 { node = writer; port = 0 }) with reads = true }
