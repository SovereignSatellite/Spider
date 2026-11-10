type live_range = { from : int; to_ : int; value : int }
type segment = { ranges : live_range array; offset : int; count : int }

val get : segment -> int -> live_range
val first_ending_after : segment -> int -> int

(** Overlapping ranges of the two must hold the same value, since they combine into one range keeping one value. *)
val merge : segment -> segment -> segment

val merge_overlapping : live_range array -> segment
