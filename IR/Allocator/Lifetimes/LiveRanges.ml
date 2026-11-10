type live_range = { from : int; to_ : int; value : int }
type segment = { ranges : live_range array; offset : int; count : int }

let get segment index = segment.ranges.(segment.offset + index)

let first_ending_after ranges position =
  let rec search low high =
    if low = high then low
    else
      let middle = (low + high) / 2 in
      if (get ranges middle).to_ <= position then search (middle + 1) high else search low middle
  in
  search 0 ranges.count

let merge large small =
  let count = large.count + small.count in
  let merged =
    if large.offset + count <= Array.length large.ranges then large
    else
      let grown = Array.make (2 * count) (get small 0) in
      Array.blit large.ranges large.offset grown 0 large.count;
      { ranges = grown; offset = 0; count = large.count }
  in
  let values = merged.ranges in
  let offset = merged.offset in
  let rec combine next range =
    if next + 1 < count && range.to_ > values.(offset + next + 1).from then
      combine (next + 1) { range with to_ = max range.to_ values.(offset + next + 1).to_ }
    else begin
      values.(offset + next) <- range;
      next - 1
    end
  in
  let rec place next kept remaining =
    if remaining > 0 && kept > 0 && values.(offset + kept - 1).from > (get small (remaining - 1)).from then
      place (combine next values.(offset + kept - 1)) (kept - 1) remaining
    else if remaining > 0 then place (combine next (get small (remaining - 1))) kept (remaining - 1)
    else if kept > 0 then place (combine next values.(offset + kept - 1)) (kept - 1) remaining
    else next
  in
  let first = place (count - 1) large.count small.count + 1 in
  { ranges = values; offset = offset + first; count = count - first }

let merge_overlapping values =
  Array.stable_sort (fun left right -> compare left.from right.from) values;
  let rec merge last index =
    if index = Array.length values then last
    else if values.(index).from <= values.(last).to_ then begin
      values.(last) <- { (values.(last)) with to_ = max values.(last).to_ values.(index).to_ };
      merge last (index + 1)
    end
    else begin
      values.(last + 1) <- values.(index);
      merge (last + 1) (index + 1)
    end
  in
  { ranges = values; offset = 0; count = merge 0 1 + 1 }
