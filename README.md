# VDL_2026 - Vienna Definition Language 2026

**IMPLICIT NONE for kernel concurrency.**

A Modern Successor to the Vienna Definition Language for Formal Specification and State-Space Exploration.

```
Where implicit assumptions become explicit specifications.
Where hidden state transitions become visible invariants.
Where subtle bugs become counterexample traces.
```

**Flagship Use Case:** Modeling RCU (Read-Copy-Update) semantics from the Linux kernel.

## IMPLICIT NONE Philosophy

Like Fortran's `IMPLICIT NONE` directive that demands explicit type declarations, VDL_2026 demands:
- Every state transition is **explicit**
- Every invariant is **checkable**
- Every assumption is **visible**
- Every violation has a **trace**

No implicit state. No hidden assumptions. No subtle bugs.

## Heritage

```
IBM Vienna Laboratory → VDL → meta-IV → VDM-SL → VDL_2026
```

VDL_2026 honors the IBM Vienna Laboratory tradition:

- **1970s: VDL** (Vienna Definition Language)
  - Used for PL/I formal semantics
  - Operational semantic metalanguage at IBM Vienna Lab
  - mk_, is_, inv_ prefixes

- **1980s: meta-IV**
  - Cleaned-up, typed successor to VDL
  - Denotational semantics core
  - Foundation for formal method research

- **1990s: VDM-SL** (Vienna Development Method Specification Language)
  - BSI standard formal specification language
  - Built on meta-IV semantics
  - Industrial-strength specification tool

- **2026: VDL_2026**
  - IMPLICIT NONE for concurrent systems
  - Explicit state semantics with operational/denotational hybrid
  - State-space exploration with invariant checking
  - Focus on Linux kernel primitives (RCU, seqlocks, etc.)

## Features

- **Denotational semantics** for types, expressions, and invariants
- **Operational semantics** for state transitions (small-step)
- **Bounded state-space exploration** (BFS/DFS)
- **Invariant checking** and violation detection with traces
- **GraphViz DOT output** for state graph visualization
- **JSON trace export** for analysis and tooling integration
- **RCU model** demonstrating kernel primitive specification

## Installation

```bash
# Clone the repository
git clone https://github.com/your-repo/vdl_2026.git
cd vdl_2026

# Build
cargo build --release

# Install (optional)
cargo install --path .
```

## Quick Start

### Explore RCU State Space

```bash
# Basic RCU exploration
vdl_2026 rcu

# With custom parameters
vdl_2026 rcu --depth 8 --readers 3 --callbacks 2

# Generate visualization
vdl_2026 rcu --graph rcu.dot
dot -Tsvg rcu.dot > rcu.svg

# Export JSON trace for analysis
vdl_2026 rcu --json trace.json
```

### View RCU Model Source

```bash
vdl_2026 show-rcu
```

### Parse a VDL_2026 Specification

```bash
vdl_2026 parse myspec.vdl
```

### Display Heritage Information

```bash
vdl_2026 info
```

## VDL_2026 Syntax

VDL_2026 uses a modern ML/TypeScript-inspired syntax while maintaining semantic rigor from the Vienna tradition.

The syntax emphasizes **explicit state** — every transition, every invariant, every assumption must be visible.

### Type Definitions (Denotational Domains)

```
type ReaderId = Nat

type Callback = {
    id: Nat,
    targetEpoch: Nat
}

type RCU = {
    readers: Set<Nat>,
    epoch: Nat,
    gpActive: Bool,
    pendingCallbacks: List<Callback>,
    completedCallbacks: List<Callback>
}
```

### Invariants

```
inv callback_safety(s: RCU) =>
    forall cb in s.pendingCallbacks:
        cb.targetEpoch >= s.epoch
```

### Transitions (Operational Semantics)

```
transition rcu_read_lock(r: Nat, s: RCU) -> RCU
    pre !member(r, s.readers)
    post s' =>
        s' == s with [readers = s.readers union {r}]

transition start_grace_period(s: RCU) -> RCU
    pre !s.gpActive
    post s' =>
        s' == s with [gpActive = true, epoch = s.epoch + 1]

transition end_grace_period(s: RCU) -> RCU
    pre s.gpActive && isEmpty(s.readers)
    post s' =>
        s' == s with [gpActive = false]
```

### Functions

```
fun partitionCallbacks(cbs: List<Callback>, epoch: Nat) -> (List<Callback>, List<Callback>) =
    let ready = filter(cbs, cb => cb.targetEpoch <= epoch) in
    let notReady = filter(cbs, cb => cb.targetEpoch > epoch) in
    (ready, notReady)
```

## Semantics

VDL_2026 implements a **hybrid semantic approach** with **IMPLICIT NONE** philosophy:

### Denotational Core (What)
- Types define mathematical domains — **explicit structure**
- Expressions have denotational meaning — **explicit values**
- Invariants are predicates over domains — **explicit constraints**
- Functions are mathematical mappings — **explicit transformations**

### Operational Layer (How)
- Transitions are state transformers — **explicit steps**
- Pre/post conditions define valid transitions — **explicit guards**
- Small-step semantics for execution — **explicit traces**
- Trace generation for debugging — **explicit violations**

This hybrid approach is faithful to the meta-IV tradition where denotational foundations support operational rules. The IMPLICIT NONE philosophy ensures nothing is hidden.

## RCU Model

The flagship model demonstrates VDL_2026 capabilities by formalizing the Linux kernel's RCU (Read-Copy-Update) synchronization primitive:

- **Read-side critical sections**: `rcu_read_lock()`, `rcu_read_unlock()`
- **Grace periods**: `start_grace_period()`, `end_grace_period()`
- **Callbacks**: `call_rcu()`, `run_ready_callbacks()`
- **Safety invariants**: Ensure callbacks don't execute while affected readers are active

The explorer can:
- Enumerate reachable states
- Check invariants at each state
- Detect violations and produce counterexample traces
- Generate state graphs for visualization

## Use Cases

- **RCU/SRCU protocol semantics** - Model Linux kernel synchronization primitives
- **Concurrent data structure invariants** - Specify and verify concurrent containers
- **API contract verification** - Formal contracts for system APIs
- **UAPI/syscall semantics modeling** - Precise userspace ABI specifications
- **Driver/hardware register protocols** - Model device interaction sequences
- **Security policies** - Formal access control specifications

## Architecture

```
┌───────────────────┐
│  VDL_2026 Source  │  IMPLICIT NONE: Explicit specifications
└─────────┬─────────┘
          │
     ┌────▼────┐
     │  Lexer  │
     └────┬────┘
          │
     ┌────▼────┐
     │  Parser │
     └────┬────┘
          │
     ┌────▼────────┐
     │     AST     │ (Denotational structure - explicit domains)
     └────┬────────┘
          │
     ┌────▼────────┐
     │  Evaluator  │ (Expression semantics - explicit values)
     └────┬────────┘
          │
     ┌────▼────────────┐
     │  State Explorer │ (Operational semantics - explicit traces)
     └────┬────────────┘
          │
     ┌────▼──────────┐
     │   Outputs     │  IMPLICIT NONE: Explicit results
     │ - Traces      │  (every step visible)
     │ - Violations  │  (every bug caught)
     │ - DOT graphs  │  (every state shown)
     │ - JSON        │  (every detail exported)
     └───────────────┘
```

## Project Structure

```
src/
├── ast.rs        # Abstract Syntax Tree (denotational structure)
├── lexer.rs      # Tokenizer using logos
├── parser.rs     # Recursive descent parser
├── value.rs      # Runtime values
├── eval.rs       # Expression evaluator
├── explorer.rs   # State-space exploration engine
├── error.rs      # Error handling
├── lib.rs        # Library root
├── main.rs       # CLI interface
└── models/
    ├── mod.rs    # Built-in models
    └── rcu.rs    # RCU semantic model
```

## Future Directions

- **Enhanced parser** for full VDL_2026 syntax
- **VS Code extension** with Language Server Protocol (LSP)
- **Web IDE** with Monaco editor and WASM backend
- **Litmus test generation** compatible with LKMM
- **SRCU and Tasks RCU** variant models
- **Memory ordering constraints** integration
- **Symbolic execution** for deeper analysis
- **Property-based testing** generation from specs
- **mk_, is_, inv_ prefixes** for authentic Vienna-style syntax

## For Kernel Developers

VDL_2026 aims to provide:
- Executable documentation for RCU semantics
- Automated test generation from formal specs
- Counterexample traces for bug reproduction
- Clear invariant specifications that can be checked

The state-space explorer can help:
- Verify that proposed RCU API changes preserve safety
- Generate minimal reproducing traces for bugs
- Cross-check against LKMM memory model
- Document subtle concurrency constraints

## Contributing

Contributions welcome! Areas of interest:
- Additional kernel primitive models (seqlocks, refcounts)
- Parser improvements and error recovery
- IDE tooling and language server
- Documentation and examples
- Performance optimizations for large state spaces

## Acknowledgments

This project honors the legacy of the IBM Vienna Laboratory and the formal methods community that developed:
- The original Vienna Definition Language
- The meta-IV metalanguage
- The Vienna Development Method

VDL_2026 is a modern continuation of this mathematical tradition, bringing executable formal specifications with IMPLICIT NONE philosophy to contemporary systems programming challenges.

## License

MIT License

---

*"For the Vienna Lab alumni: IMPLICIT NONE lives again."*

```
VDL_2026: No implicit state. No hidden assumptions. No subtle bugs.
```
