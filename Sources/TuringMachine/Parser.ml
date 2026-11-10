open Instruction

type syntax_error = UnmatchedOpen of int | UnmatchedClose of int
type sequence = Ended of Instruction.t list | Closed of Instruction.t list * int | Failed of syntax_error

let rec parse_sequence source position reversed =
  if position = String.length source then Ended (List.rev reversed)
  else
    let next = position + 1 in
    match source.[position] with
    | '>' -> parse_sequence source next (MoveRight :: reversed)
    | '<' -> parse_sequence source next (MoveLeft :: reversed)
    | '+' -> parse_sequence source next (Increment :: reversed)
    | '-' -> parse_sequence source next (Decrement :: reversed)
    | '.' -> parse_sequence source next (Output :: reversed)
    | ',' -> parse_sequence source next (Input :: reversed)
    | '[' -> parse_loop source position reversed
    | ']' -> Closed (List.rev reversed, next)
    | _ -> parse_sequence source next reversed

and parse_loop source position reversed =
  match parse_sequence source (position + 1) [] with
  | Closed (body, after_body) -> parse_sequence source after_body (Loop body :: reversed)
  | Ended _ -> Failed (UnmatchedOpen position)
  | Failed error -> Failed error

let parse source =
  match parse_sequence source 0 [] with
  | Ended instructions -> Ok instructions
  | Closed (_, next) -> Error (UnmatchedClose (next - 1))
  | Failed error -> Error error
