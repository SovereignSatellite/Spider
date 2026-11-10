(** Boissinot et al. 2009's Algorithm 1, but swapping along a cycle instead of copying through a spare register. Its
    cycle test on line 18, [b = loc(pred(b))], is true only once the move into [b] is made, too late, so this one tests
    [loc(b) = b]. *)

val sequentialize : (Register.t * Register.t * IR.Type.t) list -> 'x Move.t list
