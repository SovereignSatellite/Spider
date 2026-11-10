(** The root's one parameter, the start, is a [Unit] no node produces: whatever must be ordered depends on it, since a
    [Unit] constant waits for nothing. *)
type 'x t = { types : 'x TypeArena.t; root : 'x Region.t }
