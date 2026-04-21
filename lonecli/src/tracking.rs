use core::sync::atomic::{AtomicU8, AtomicU32, AtomicUsize, Ordering};

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
    max_visible: AtomicU32,
    sure_win_hits: AtomicUsize,

    // Branching factor (post-pruner): sum and max of n_moves per expansion.
    branching_sum: AtomicUsize,
    branching_max: AtomicU8,
    dead_end_count: AtomicUsize,

    // Backtracks counted in finish_move.
    backtracks: AtomicUsize,

    // Pruner effectiveness: accumulates move counts before and after FullPruner.
    prune_total_unfiltered: AtomicUsize,
    prune_total_filtered: AtomicUsize,
}

impl Default for AtomicSearchStats {
    fn default() -> Self {
        Self {
            total_visit: AtomicUsize::new(0),
            unique_visit: AtomicUsize::new(0),
            max_depth: AtomicUsize::new(0),
            move_state: Default::default(),
            max_stack_reached: AtomicU8::new(0),
            min_hidden_down: AtomicU8::new(u8::MAX),
            max_visible: AtomicU32::new(0),
            sure_win_hits: AtomicUsize::new(0),
            branching_sum: AtomicUsize::new(0),
            branching_max: AtomicU8::new(0),
            dead_end_count: AtomicUsize::new(0),
            backtracks: AtomicUsize::new(0),
            prune_total_unfiltered: AtomicUsize::new(0),
            prune_total_filtered: AtomicUsize::new(0),
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

    #[must_use]
    pub(crate) fn max_visible(&self) -> u32 {
        self.max_visible.load(Ordering::Relaxed)
    }

    #[must_use]
    pub(crate) fn sure_win_hits(&self) -> usize {
        self.sure_win_hits.load(Ordering::Relaxed)
    }

    #[must_use]
    pub(crate) fn branching_avg(&self) -> f64 {
        let sum = self.branching_sum.load(Ordering::Relaxed);
        let n = self.unique_visit.load(Ordering::Relaxed);
        if n == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                sum as f64 / n as f64
            }
        }
    }

    #[must_use]
    pub(crate) fn branching_max(&self) -> u8 {
        self.branching_max.load(Ordering::Relaxed)
    }

    #[must_use]
    pub(crate) fn dead_end_count(&self) -> usize {
        self.dead_end_count.load(Ordering::Relaxed)
    }

    #[must_use]
    pub(crate) fn backtracks(&self) -> usize {
        self.backtracks.load(Ordering::Relaxed)
    }

    // Fraction of moves eliminated by the FullPruner on top of gen_moves::<true>'s dominance rules.
    #[must_use]
    pub(crate) fn prune_rate(&self) -> f64 {
        let unfiltered = self.prune_total_unfiltered.load(Ordering::Relaxed);
        let filtered = self.prune_total_filtered.load(Ordering::Relaxed);
        if unfiltered == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                (unfiltered - filtered) as f64 / unfiltered as f64
            }
        }
    }
}

impl SearchStatistics for AtomicSearchStats {
    fn hit_a_state(&self, depth: usize) {
        self.max_depth.fetch_max(depth, Ordering::Relaxed);
        self.total_visit.fetch_add(1, Ordering::Relaxed);
    }

    fn hit_unique_state(&self, depth: usize, n_moves: u32) {
        self.unique_visit.fetch_add(1, Ordering::Relaxed);

        #[allow(clippy::cast_possible_truncation)]
        {
            self.branching_sum
                .fetch_add(n_moves as usize, Ordering::Relaxed);
            self.branching_max
                .fetch_max(n_moves.min(u32::from(u8::MAX)) as u8, Ordering::Relaxed);
        }
        if n_moves == 0 {
            self.dead_end_count.fetch_add(1, Ordering::Relaxed);
        }

        if depth < TRACK_DEPTH {
            self.move_state[depth].0.store(0, Ordering::Relaxed);
            self.move_state[depth]
                .1
                .store(n_moves.min(u32::from(u8::MAX)) as u8, Ordering::Relaxed);
        }
    }

    fn finish_move(&self, depth: usize) {
        self.backtracks.fetch_add(1, Ordering::Relaxed);
        if depth < TRACK_DEPTH {
            self.move_state[depth].0.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn hit_game_state(&self, stack_len: u8, hidden_down: u8, deck_len: u8, visible: u32) {
        self.max_stack_reached
            .fetch_max(stack_len, Ordering::Relaxed);
        self.min_hidden_down
            .fetch_min(hidden_down, Ordering::Relaxed);
        self.max_visible.fetch_max(visible, Ordering::Relaxed);
        // Matches Solitaire::is_sure_win(): deck.len() <= 1 && hidden.is_all_up() (i.e. no down cards).
        if deck_len <= 1 && hidden_down == 0 {
            self.sure_win_hits.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn hit_pruner_info(&self, unfiltered: u32, filtered: u32) {
        self.prune_total_unfiltered
            .fetch_add(unfiltered as usize, Ordering::Relaxed);
        self.prune_total_filtered
            .fetch_add(filtered as usize, Ordering::Relaxed);
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
            "Total visit: {}\nTransposition hit: {} (rate {})\nMiss state: {}\nMax depth search: {}\nBacktracks: {}\nBranching (post-prune): avg {:.2}, max {}, dead-ends {}\nPruner reduction (post-dominance): {:.2}%\nMax foundation reached: {}/52\nMin hidden-cards seen: {}\nMax visible cards: {}/45\nSure-win states visited: {}\nCurrent progress:",
            total,
            hit,
            (hit as f64) / (total as f64),
            unique,
            depth,
            self.backtracks(),
            self.branching_avg(),
            self.branching_max(),
            self.dead_end_count(),
            self.prune_rate() * 100.0,
            self.max_stack_reached(),
            if min_hidden == u8::MAX {
                String::from("-")
            } else {
                min_hidden.to_string()
            },
            self.max_visible(),
            self.sure_win_hits(),
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
