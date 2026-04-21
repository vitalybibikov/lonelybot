use core::time::Duration;
use lonelybot::{
    graph::{graph_with_tracking, Graph},
    solver::{solve_with_tp, HistoryVec, SearchResult},
    state::Solitaire,
    tracking::TerminateSignal,
    traverse::{Control, TpTable},
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, RecvTimeoutError},
        Arc,
    },
    thread,
};

use crate::tracking::AtomicSearchStats;

const STACK_SIZE: usize = 4 * 1024 * 1024;

struct TermSignal<'a> {
    term_signal: &'a AtomicBool,
}

impl TerminateSignal for TermSignal<'_> {
    fn terminate(&self) {
        self.term_signal.store(true, Ordering::Relaxed);
    }

    fn is_terminated(&self) -> bool {
        self.term_signal.load(Ordering::Relaxed)
    }
}

pub(crate) fn run_solve(
    mut g: Solitaire,
    verbose: bool,
    term_signal: &Arc<AtomicBool>,
) -> (SearchResult, AtomicSearchStats, Option<HistoryVec>, usize) {
    let ss = Arc::new(AtomicSearchStats::new());

    let (send, recv) = channel::<()>();

    let child = {
        // Spawn thread with explicit stack size
        let ss_clone = ss.clone();
        let term = term_signal.clone();
        thread::Builder::new()
            .stack_size(STACK_SIZE)
            .spawn(move || {
                let mut tp = TpTable::default();
                let res = solve_with_tp(
                    &mut g,
                    ss_clone.as_ref(),
                    &TermSignal {
                        term_signal: term.as_ref(),
                    },
                    &mut tp,
                );
                let tp_len = tp.len();
                send.send(()).ok();
                (res, tp_len)
            })
            .unwrap()
    };

    if verbose {
        loop {
            match recv.recv_timeout(Duration::from_millis(1000)) {
                Err(RecvTimeoutError::Disconnected) | Ok(()) => break,
                Err(RecvTimeoutError::Timeout) => println!("{ss}"),
            }
        }
    }

    let ((res, hist), tp_len) = child
        .join()
        .unwrap_or(((SearchResult::Crashed, None), 0));

    (res, Arc::try_unwrap(ss).unwrap(), hist, tp_len)
}

pub(crate) fn run_graph(
    mut g: Solitaire,
    verbose: bool,
    term_signal: &Arc<AtomicBool>,
) -> (Option<(Control, Graph)>, AtomicSearchStats) {
    let ss = Arc::new(AtomicSearchStats::new());

    let (send, recv) = channel::<()>();

    let child = {
        // Spawn thread with explicit stack size
        let ss_clone = ss.clone();
        let term = term_signal.clone();
        thread::Builder::new()
            .stack_size(STACK_SIZE)
            .spawn(move || {
                let res = graph_with_tracking(
                    &mut g,
                    ss_clone.as_ref(),
                    &TermSignal {
                        term_signal: term.as_ref(),
                    },
                );
                send.send(()).ok();
                res
            })
            .unwrap()
    };

    if verbose {
        loop {
            match recv.recv_timeout(Duration::from_millis(1000)) {
                Err(RecvTimeoutError::Disconnected) | Ok(()) => break,
                Err(RecvTimeoutError::Timeout) => println!("{ss}"),
            }
        }
    }

    let res = child.join().ok();
    (res, Arc::try_unwrap(ss).unwrap())
}
