# CLAUDE.md — VRP Solver Project

## Project Overview

This is a production-grade Rust reimplementation of **AILS-II** (Máximo, Cordeau & Nascimento, INFORMS JoC 2024) for the Capacitated Vehicle Routing Problem (CVRP), with planned extension to generic VRP variants. The reference Java implementation is at https://github.com/vinymax10/AILS-CVRP.

The project is developed as part of a structured 38-week Rust learning curriculum. The author is an experienced scientist/engineer (PhD CS, 15+ years OR/optimisation research, production Python+Gurobi systems) learning Rust from scratch with a production goal: replace Python bottlenecks with a Rust core callable from Python via PyO3/maturin bindings.

**Benchmark instances:** Uchoa et al. X-instances and Arnold et al. instances (in `data/`).  
**Quality target:** Within 1% of Best Known Solutions (BKS) on X-E instances (n≤100) at Phase 4 gate.

---

## Architecture — 6-Layer Design

All code must respect this layered architecture. No circular dependencies between layers.

```
driver/          ← Entry point, CLI, orchestration (uses solver)
solver/          ← Orchestrates algorithmic operations
├── algorithmic/ ← Metaheuristic components (AILS-II loop, perturbation, LS)
├── solution/    ← Solution representation, route structures, delta evaluation
├── problem/     ← Problem components (constraints, feasibility, cost)
├── preparation/ ← Data preparation (distance matrix, neighbor lists, granularity)
└── loading/     ← Data loading, instance parsing, config parsing
```

**Rules:**
- `loading` and `preparation` have no dependencies on upper layers
- `algorithmic` depends on `solution` and `problem` traits only — never on concrete types from other layers
- `driver` may use `anyhow` for error handling; all library layers must use `thiserror` with typed errors
- No `println!` in library code — use `tracing` spans and events throughout

---

## AILS-II Algorithm Reference

Key parameters (from Máximo et al. 2024):

| Parameter | Default | Description |
|-----------|---------|-------------|
| `varphi`  | 40      | Max cardinality of nearest-neighbor set per vertex |
| `gamma`   | 30      | Iterations between ω adjustment cycles |
| `dMax`    | 30      | Initial reference perturbation distance |
| `dMin`    | 15      | Final reference perturbation distance |

Stopping criteria: `Time` (seconds) or `Iteration` (count) — both must be supported as config options.

The ω parameter controls the balance between exploitation and exploration. Its adaptive adjustment is the key differentiator of AILS-II over vanilla ILS. When reviewing algorithmic code, verify the ω adaptation logic matches the paper's description.

---

## Coding Standards

These are non-negotiable at all phases:

- **No `unwrap()` or `expect()` in library code.** Propagate errors with `?`. Panics are only acceptable in `driver` for unrecoverable startup failures, and must be documented.
- **No unnecessary `clone()`.** If you see a clone, question it. Either it's justified (document why) or it's hiding a borrowing problem.
- **No raw index loops where iterators suffice.** Prefer iterator chains. Index loops are acceptable only in hot-path delta evaluation where bounds checks matter.
- **No `Box<dyn Trait>` without justification.** Prefer generics with trait bounds. Dynamic dispatch is acceptable at the driver/solver boundary where monomorphisation cost is not worth it.
- **All public functions must be documented** with `///` doc comments. `cargo doc` must build without warnings.
- **`cargo clippy -- -D warnings` must pass clean** on every commit. No exceptions.
- **`cargo fmt`** must be applied before every commit.
- **Tests:** Unit tests in the same file (`#[cfg(test)]`). Integration tests in `tests/`. Every public function in `problem/` and `solution/` layers must have unit tests.

---

## Curriculum Phases & Review Criteria

### Phase 1 — Foundations (Weeks 1–8)
*Goal: Read and write idiomatic Rust. Understand ownership. Build data loading layer skeleton.*

| Weeks | Topic | Review Criteria |
|-------|-------|----------------|
| 1–2 | Toolchain, cargo, syntax, control flow | Correct cargo project structure · No clippy warnings · Unit tests pass |
| 3–4 | Ownership, borrowing, slices | No unnecessary clones · Lifetimes implicit but correct · No raw indexing panics |
| 5–6 | Structs, enums, pattern matching, Option/Result | Exhaustive match · No `unwrap()` in library code · Custom error type |
| 7–8 | Collections, iterators, closures | Iterator chains not index loops · HashMap keyed correctly · Integration test with benchmark instance |

**Phase 1 Gate:** Data loading layer complete and tested. Parses `.vrp` instance files. `Customer`, `Instance`, and distance matrix structs exist. All clippy clean.

---

### Phase 2 — Intermediate Rust (Weeks 9–16)
*Goal: Generic, trait-bound code. 6-layer architecture established. Solution representation layer built.*

| Weeks | Topic | Review Criteria |
|-------|-------|----------------|
| 9–10 | Traits and generics | Trait objects vs generics distinction · No unnecessary `Box<dyn T>` · Minimal trait bounds |
| 11–12 | Module system, visibility, crate layout | `pub` API surface intentional · `cargo doc` builds clean · No circular dependencies |
| 13–14 | Explicit lifetimes, advanced borrowing | Named lifetimes correct · No `Rc`/`RefCell` as workaround · No hidden allocations |
| 15–16 | Error handling, thiserror/anyhow, testing | `thiserror` for library errors · `anyhow` in driver only · Test coverage >80% on components |

**Phase 2 Gate:** Problem components layer complete. All public functions return `Result`. Core traits (`Problem`, `Solution`, `Move`) defined. Integration tests pass on Uchoa/benchmark instances.

---

### Phase 3 — Performance Rust (Weeks 17–22)
*Goal: Understand memory layout. Profile hot path. Python bindings skeleton working.*

| Weeks | Topic | Review Criteria |
|-------|-------|----------------|
| 17–18 | Memory layout, SoA vs AoS, cache lines | `criterion` benchmark with statistical output · Layout decision documented · No premature pessimisation |
| 19–20 | Zero-cost iterators, rayon | `rayon` used correctly · No data races · Speedup documented on N=1000 |
| 21–22 | `unsafe` basics, FFI, PyO3 hello world | PyO3 bindings compile · GIL handling correct · Rust panics don't crash Python |

**Phase 3 Gate:** Python wrapper skeleton. `cargo build --release` + `pip install -e .` working. `compute_route_cost` callable from Python.

---

### Phase 4 — Algorithmic Components (Weeks 23–32)
*Goal: AILS-II core implemented. CVRP results within 1% BKS on X-E instances.*

| Weeks | Topic | Review Criteria |
|-------|-------|----------------|
| 23–24 | Construction heuristics | No feasibility violations · O(n²) not O(n³) · Deterministic given seed |
| 25–27 | Or-opt moves, delta evaluation | Delta eval correct (unit tested) · Move abstracted behind trait · ≥10k moves/sec on n=100 |
| 28–29 | Granularity filter (FILO2-style) | Neighbor lists pre-computed · k=20 matches literature quality · No regression on small instances |
| 30–32 | AILS-II loop, ω adaptation, perturbation | Results table vs BKS · Reproducible with seed · Solution writer outputs valid JSON/CSV · Python callable |

**Phase 4 Gate:** Full AILS-II on CVRP. Uchoa X-E instances within 1% BKS. Seed-reproducible. Python-callable. Solution writer complete.

---

### Phase 5 — Production Hardening (Weeks 33–38)
*Goal: Shippable library. pip-installable, documented, CI-gated.*

| Weeks | Topic | Review Criteria |
|-------|-------|----------------|
| 33–34 | CLI (clap), config (serde/toml), tracing | `clap` derive API · Config validated at startup · Structured JSON logs · No `println!` in library |
| 35–36 | maturin packaging, GitHub Actions CI | CI passes on clean checkout · Wheel installable on Linux/macOS · Python API documented |
| 37–38 | VRPTW extension | Zero changes to algorithmic layer · New `Problem` impl only · Solomon C1 within 2% BKS |

**Phase 5 Gate:** VRPTW variant working. CI green. Wheel on TestPyPI. Demonstrates generic architecture.

---

## Review Protocol

When asked to review a branch for merging into `main`, follow this process:

### 1. Identify the Phase and Week
From the branch name (`phase{N}-week{W}-{topic}`) or by asking. This determines which criteria apply.

### 2. Run Checks First
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
Report results before reviewing code. A clippy failure or test failure is a hard block — do not proceed to qualitative review until clean.

### 3. Review Against Phase Criteria
Evaluate each criterion for the relevant week as: **Pass**, **Fail**, or **Needs Work** (minor issue, fixable without full rework).

### 4. Feedback Categories
Structure feedback under these four headings:

- **Correctness** — Does it do what it's supposed to? Are tests adequate? Are edge cases handled?
- **Idiomatic Rust** — Ownership patterns, iterator usage, error handling, naming conventions (snake_case, descriptive). Would a senior Rust engineer be comfortable reading this?
- **Performance** — Unnecessary allocations, clones, missed zero-cost opportunities. Flag but don't over-optimise before Phase 3.
- **Architecture** — Does it respect the 6-layer boundaries? Is the `pub` surface correct? Does it compose well with the rest of the project?

### 5. Gate Decision
- **Approve:** All phase criteria met. Minor notes can be addressed in follow-up.
- **Request Changes:** One or more criteria failed. List exactly what must be fixed before merge.
- **Reject:** Fundamental structural issue that would require rework to fix later. Explain why and suggest an approach.

### 6. Carry-Forward Tracking
If the same issue appears in two consecutive reviews (e.g. repeated `unwrap()` use, repeated clippy suppressions), flag it explicitly as a pattern and suggest a focused drill to address it before the next submission.

---

## Key References

- **AILS-II paper:** Máximo, Cordeau & Nascimento (2024). AILS-II: An Adaptive Iterated Local Search Heuristic for the Large-Scale CVRP. *INFORMS Journal on Computing*. https://doi.org/10.1287/ijoc.2023.0106
- **AILS-II repo (Java reference):** https://github.com/vinymax10/AILS-CVRP
- **FILO2 (granularity reference):** Accorsi & Vigo (2021)
- **Benchmark instances:** Uchoa et al. X-instances · Arnold et al.
- **The Rust Book:** https://doc.rust-lang.org/book/
- **Rust API Guidelines:** https://rust-lang.github.io/api-guidelines/
- **PyO3 Guide:** https://pyo3.rs/
- **maturin:** https://www.maturin.rs/
- **criterion.rs:** https://bheisler.github.io/criterion.rs/book/

---

## Notes on the Author's Background

The developer is an expert in combinatorial optimisation and metaheuristics, not a Rust novice in the general CS sense. Reviews should:

- **Not** over-explain algorithmic concepts — focus on Rust-specific issues
- **Do** be direct about Rust idioms and ownership patterns, which are the genuine learning frontier
- **Do** flag performance issues even in early phases if they would require structural rework later
- **Not** suggest over-engineering — pragmatic, production-quality code is the goal, not academic Rust showcasing
