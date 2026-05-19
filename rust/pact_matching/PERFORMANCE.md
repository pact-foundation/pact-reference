# Performance Optimisation Guide

This document describes the process used to profile and optimise the V2 matching
engine, and can be followed for future performance work.

## Tools

| Tool | Purpose |
|---|---|
| `cargo flamegraph` | CPU profiling — identifies *where* time is spent |
| `criterion` | Statistical benchmarking — measures *how fast* and tracks regressions |
| `heaptrack` | Allocation profiling — identifies *what* is being allocated and from where |

Install flamegraph if not already present:

```bash
cargo install flamegraph
```

Flamegraph uses `perf` on Linux. If it complains about permissions:

```bash
echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
```

## The Workflow

The process follows a repeated cycle:

```
flamegraph → identify hotspot → benchmark baseline → fix → benchmark → flamegraph → ...
```

**Always benchmark before making a change.** You need a "before" number to compare
against. Without it you know the code is different but not whether it's better.

## Step 1 — Generate a Flamegraph

The profiling build profile in `rust/Cargo.toml` gives release-speed optimisations
with debug symbols retained (required for readable stack frames):

```toml
[profile.profiling]
inherits = "release"
strip = false
debug = true
```

The profiling target is `pact_matching/examples/profile_engine.rs`. It runs four
representative matching scenarios (text body, JSON equality, JSON with matching
rules, headers + query parameters) in a tight loop so `perf` collects enough samples.

Generate the flamegraph from the `rust/` directory:

```bash
cargo flamegraph \
  --package pact_matching \
  --example profile_engine \
  --profile profiling \
  --output flamegraph.svg
```

Open `flamegraph.svg` in a browser. The SVG is interactive — click any frame to
zoom in. The widest bars are the hotspots.

### Reading the flamegraph

- **X-axis** — time (width = percentage of CPU samples)
- **Y-axis** — call stack depth (bottom = caller, top = callee/leaf)
- **`[unknown]`** — symbol could not be resolved; usually allocator internals
  (glibc/ptmalloc) or Rust stdlib code compiled without debug symbols. Look at
  the frames *above* it (callees) and *below* it (callers) to understand context.
- When `[unknown]` sits below a `clone` frame, the flamegraph has hit its symbol
  resolution limit. Switch to `heaptrack` for allocation-specific analysis.

## Step 2 — Establish a Benchmark Baseline

Benchmarks live in `pact_matching/benches/v2_engine.rs` and use
[criterion](https://github.com/bheisler/criterion.rs).

Clear any prior criterion data first (stale baselines cause misleading comparisons):

```bash
rm -rf rust/target/criterion
```

Run the full benchmark suite from the `rust/` directory:

```bash
cargo bench --package pact_matching --bench v2_engine
```

Criterion saves results under `target/criterion/`. The next run after a code change
will automatically compare against this baseline and print a change percentage with
statistical confidence.

For a quick smoke-run during development (faster, less accurate):

```bash
cargo bench --package pact_matching --bench v2_engine \
  -- --warm-up-time 1 --measurement-time 1 --sample-size 10
```

### Adding new benchmarks

Each benchmark group covers one scenario split into three phases:

- **`build`** — plan construction only
- **`execute`** — plan execution only (pre-built plan, measures interpreter cost)
- **`build+execute`** — end-to-end (what callers actually pay)

Separating build and execute makes it easy to see which phase a change affects.

## Step 3 — Identify and Fix the Hotspot

With a flamegraph showing where time goes and a criterion baseline to measure
against, make the change and re-run the benchmarks:

```bash
cargo bench --package pact_matching --bench v2_engine
```

Criterion will print lines like:

```
json_equality/execute   time:   [153.21 µs 153.63 µs 154.10 µs]
                        change: [-35.539% -35.374% -35.215%] (p = 0.00 < 0.05)
                        Performance has improved.
```

If the change is worthwhile, generate a new flamegraph to find the next hotspot.
Repeat the cycle.

## Step 4 — Know When to Stop

Benchmark numbers in isolation are not enough — consider the real-world call rate.

The matching engine is invoked once per Pact interaction: perhaps a few dozen times
in a typical test run, up to ~100–200 for large Pact files. At that scale, even a
200 µs execution time contributes only ~40 ms total, which is negligible compared
to network I/O or mock server startup. Further micro-optimisation below that
threshold has no practical impact on test suite runtime.

Stop when:
- The remaining hotspots are in allocator internals with no clear code-level fix
- The absolute time per call × realistic call rate is already imperceptible
- The complexity of the next fix outweighs the gain

## History

### 2026-05 — Eliminate redundant tree-cloning in the interpreter

**Finding:** `cargo flamegraph` showed `malloc` and `clone` as the dominant
hotspots. The interpreter built a complete shadow tree on every execution by
cloning every node of the plan tree.

**Fix:** Changed `walk_tree` and all `execute_*` methods to take `ExecutionPlanNode`
by value instead of by reference. Nodes are now moved/transformed into the executed
tree rather than cloned. The public API (`execute_request_plan` etc.) was preserved
by cloning `plan_root` once at the entry point.

**Result:** 30–35% reduction in execution time across all scenarios.
