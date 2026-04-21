pub trait SearchStatistics {
    fn hit_a_state(&self, depth: usize);
    fn hit_unique_state(&self, depth: usize, n_moves: u32);
    fn finish_move(&self, depth: usize);

    // Game-state snapshot on every visit. Default no-op preserves all existing impls.
    fn hit_game_state(&self, _stack_len: u8, _hidden_down: u8) {}
}

pub struct EmptySearchStats;

impl SearchStatistics for EmptySearchStats {
    fn hit_a_state(&self, _: usize) {}
    fn hit_unique_state(&self, _: usize, _: u32) {}
    fn finish_move(&self, _: usize) {}
}

pub trait TerminateSignal {
    fn terminate(&self) {}
    fn is_terminated(&self) -> bool {
        false
    }
}

pub struct DefaultTerminateSignal;

impl TerminateSignal for DefaultTerminateSignal {}
