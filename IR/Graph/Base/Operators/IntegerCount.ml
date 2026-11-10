type t = LeadingZeros | TrailingZeros | CountOnes

let name operator =
  match operator with LeadingZeros -> "LeadingZeros" | TrailingZeros -> "TrailingZeros" | CountOnes -> "CountOnes"
