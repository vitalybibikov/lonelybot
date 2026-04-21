use crate::{
    moves::Move,
    pruning::FullPruner,
    state::{Encode, Solitaire},
    tracking::{DefaultTerminateSignal, EmptySearchStats, SearchStatistics, TerminateSignal},
    traverse::{traverse, Callback, Control, TpTable},
};
use arrayvec::ArrayVec;

// before every progress you'd do at most 2*N_RANKS move
// and there would only be N_FULL_DECK + N_HIDDEN progress step
const N_PLY_MAX: usize = 1024;

pub type HistoryVec = ArrayVec<Move, N_PLY_MAX>;

#[derive(Debug, PartialEq, Eq)]
pub enum SearchResult {
    Terminated,
    Solved,
    Unsolvable,
    Crashed,
}

struct SolverCallback<'a, S: SearchStatistics, T: TerminateSignal> {
    history: HistoryVec,
    stats: &'a S,
    sign: &'a T,
    result: SearchResult,
}

impl<S: SearchStatistics, T: TerminateSignal> Callback for SolverCallback<'_, S, T> {
    type Pruner = FullPruner;
    fn on_win(&mut self, _: &Solitaire) -> Control {
        self.result = SearchResult::Solved;
        Control::Halt
    }

    fn on_visit(&mut self, game: &Solitaire, _: Encode) -> Control {
        if self.sign.is_terminated() {
            self.result = SearchResult::Terminated;
            return Control::Halt;
        }

        self.stats.hit_a_state(self.history.len());
        self.stats
            .hit_game_state(game.get_stack().len(), game.get_hidden().total_down_cards());
        Control::Ok
    }

    fn on_move_gen(&mut self, m: &crate::moves::MoveMask, _: Encode) -> Control {
        self.stats.hit_unique_state(self.history.len(), m.len());
        Control::Ok
    }

    fn on_do_move(&mut self, _: &Solitaire, m: Move, _: Encode, _: &FullPruner) -> Control {
        self.history.push(m);
        Control::Ok
    }

    fn on_undo_move(&mut self, _: Move, _: Encode, res: &Control) {
        if *res == Control::Ok {
            self.history.pop();
        }
        self.stats.finish_move(self.history.len());
    }
}

pub fn solve_with_tracking<S: SearchStatistics, T: TerminateSignal>(
    game: &mut Solitaire,
    stats: &S,
    sign: &T,
) -> (SearchResult, Option<HistoryVec>) {
    let mut tp = TpTable::default();
    solve_with_tp(game, stats, sign, &mut tp)
}

// Same as solve_with_tracking but accepts a caller-owned transposition table,
// so the caller can inspect its size after the search completes.
pub fn solve_with_tp<S: SearchStatistics, T: TerminateSignal>(
    game: &mut Solitaire,
    stats: &S,
    sign: &T,
    tp: &mut TpTable,
) -> (SearchResult, Option<HistoryVec>) {
    let mut callback = SolverCallback {
        history: HistoryVec::new(),
        stats,
        sign,
        result: SearchResult::Unsolvable,
    };

    traverse(game, FullPruner::default(), tp, &mut callback);

    let result = callback.result;

    if result == SearchResult::Solved {
        (result, Some(callback.history))
    } else {
        (result, None)
    }
}

pub fn solve(game: &mut Solitaire) -> (SearchResult, Option<HistoryVec>) {
    solve_with_tracking(game, &EmptySearchStats {}, &DefaultTerminateSignal {})
}
