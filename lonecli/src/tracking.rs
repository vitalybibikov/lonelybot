use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

use lonelybot::tracking::SearchStatistics;

const TRACK_DEPTH: usize = 8;

#[derive(Debug)]
pub(crate) struct AtomicSearchStats {
    total_visit: AtomicUsize,
    unique_visit: AtomicUsize,
    max_depth: AtomicUsize,
    move_state: [(AtomicU8, AtomicU8); TRACK_DEPTH],
    max_stack_reached: AtomicU8,
    min_hidden_down: AtomicU8,
}

impl Default for AtomicSearchStats {
    fn default() -> Self {
        Self {
            total_visit: AtomicUsize::new(0),
            unique_visit: AtomicUsize::new(0),
            max_depth: AtomicUsize::new(0),
            move_state: Default::default(),
            max_stack_reached: AtomicU8::new(0),
            // Hidden cards in a fresh deal = 21 (rows 1..6, 7*(7+1)/2 - 7). Start above any possible value.
            min_hidden_down: AtomicU8::new(u8::MAX),
        }
    }
}

impl AtomicSearchStats {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub(crate) fn total_visit(&self) -> usize {
        self.total_visit.load(Ordering::Relaxed)
    }

    #[must_use]
    pub(crate) fn unique_visit(&self) -> usize {
        self.unique_visit.load(Ordering::Relaxed)
    }

    #[must_use]
    pub(crate) fn max_depth(&self) -> usize {
        self.max_depth.load(Ordering::Relaxed)
    }

    #[must_use]
    pub(crate) fn max_stack_reached(&self) -> u8 {
        self.max_stack_reached.load(Ordering::Relaxed)
    }

    #[must_use]
    pub(crate) fn min_hidden_down(&self) -> u8 {
        self.min_hidden_down.load(Ordering::Relaxed)
    }
}

impl SearchStatistics for AtomicSearchStats {
    fn hit_a_state(&self, depth: usize) {
        self.max_depth.fetch_max(depth, Ordering::Relaxed);
        self.total_visit.fetch_add(1, Ordering::Relaxed);
    }

    fn hit_unique_state(&self, depth: usize, n_moves: u32) {
        self.unique_visit.fetch_add(1, Ordering::Relaxed);

        if depth < TRACK_DEPTH {
            self.move_state[depth].0.store(0, Ordering::Relaxed);
            self.move_state[depth]
                .1
                .store(n_moves as u8, Ordering::Relaxed);
        }
    }

    fn finish_move(&self, depth: usize) {
        if depth < TRACK_DEPTH {
            self.move_state[depth].0.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn hit_game_state(&self, stack_len: u8, hidden_down: u8) {
        self.max_stack_reached
            .fetch_max(stack_len, Ordering::Relaxed);
        self.min_hidden_down
            .fetch_min(hidden_down, Ordering::Relaxed);
    }
}

impl core::fmt::Display for AtomicSearchStats {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (total, unique, depth) = (self.total_visit(), self.unique_visit(), self.max_depth());
        let hit = total - unique;
        let min_hidden = self.min_hidden_down();
        #[allow(clippy::cast_precision_loss)]
        write!(
            f,
            "Total visit: {}\nTransposition hit: {} (rate {})\nMiss state: {}\nMax depth search: {}\nMax foundation reached: {}/52\nMin hidden-cards seen: {}\nCurrent progress:",
            total,
            hit,
            (hit as f64) / (total as f64),
            unique,
            depth,
            self.max_stack_reached(),
            if min_hidden == u8::MAX {
                String::from("-")
            } else {
                min_hidden.to_string()
            },
        )?;

        for (cur, total) in &self.move_state {
            write!(
                f,
                " {}/{}",
                cur.load(Ordering::Relaxed),
                total.load(Ordering::Relaxed)
            )?;
        }
        Ok(())
    }
}
