# Fork Instructions — difficulty metrics for lonelybot

## What was added
Two new `lonecli` subcommands and six new signal-bearing metrics on top of upstream's solver output:
- `lonecli playout-rate <seed_type> <seed> <draw_step> <trials>` — random-rollout win probability + 95% Wilson CI
- `lonecli features  <seed_type> <seed> <draw_step>` — 14 a priori deal features computed from the initial shuffle, no search
- `lonecli solve ...` now also prints: `Branching avg / max / dead-ends`, `Pruner reduction %`, `Max foundation / Min hidden / Max visible`, `Sure-win states`, `Transposition table size`, `States/sec`, and a solution move-type histogram

## Why this matters
Upstream's `solve` output was 8 numbers, most of which don't correlate with real-player difficulty. On a 249-seed production dataset with real P50 scores, we empirically ranked every metric by Spearman correlation. The winners that distinguish easy vs. hard seeds:
- `playout_rate` (+0.327) — best non-solver signal, ~5 ms per seed
- `tp_hit_rate` (-0.347) — best solver signal (already existed)
- `aces_buried_depth_sum` (-0.188) — best zero-cost a priori signal
- composite `playout_rate − tp_hit_rate` (+0.368) — best overall

## How to use it
Order seeds from easy to hard without running a full solve:
```
lonecli playout-rate exact <256-bit-seed> 3 1000
```
Higher fraction = easier. 1000 trials gives ~3% margin.

Rank seeds with zero search cost:
```
lonecli features exact <256-bit-seed> 3   # use aces_buried_depth_sum (lower = easier)
```

Maximum-confidence single score (needs both):
```
composite = playout_rate − tp_hit_rate      # from the two commands above; higher = easier
```

Everything else the fork prints (move histogram, max_depth, worry-back count, kings_*, color_imbalance, etc.) was measured and found to be noise on solvable games — keep for debugging, not for difficulty ranking.
