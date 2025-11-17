# VDL++ - Vienna Definition Language Plus Plus

**A Modern Successor to the Vienna Definition Language for Formal Specification and State-Space Exploration**

VDL++ combines **denotational** and **operational semantics** in the Vienna tradition for formal specification and state-space exploration of concurrent systems.

**Flagship Use Case:** Modeling RCU (Read-Copy-Update) semantics from the Linux kernel.

## Heritage

```
IBM Vienna Laboratory → VDL → meta-IV → VDM-SL → VDL++
```

VDL++ honors the IBM Vienna Laboratory tradition:

- **1970s: VDL** (Vienna Definition Language)
  - Used for PL/I formal semantics
  - Operational semantic metalanguage at IBM Vienna Lab

- **1980s: meta-IV**
  - Cleaned-up, typed successor to VDL
  - Denotational semantics core
  - Foundation for formal method research

- **1990s: VDM-SL** (Vienna Development Method Specification Language)
  - BSI standard formal specification language
  - Built on meta-IV semantics
  - Industrial-strength specification tool

- **2025: VDL++**
  - Modern hybrid operational/denotational semantics
  - State-space exploration for concurrent systems
  - Executable specifications with invariant checking
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
git clone https://github.com/your-repo/vdlpp.git
cd vdlpp

# Build
cargo build --release

# Install (optional)
cargo install --path .
```

## Quick Start

### Explore RCU State Space

```bash
# Basic RCU exploration
vdlpp rcu

# With custom parameters
vdlpp rcu --depth 8 --readers 3 --callbacks 2

# Generate visualization
vdlpp rcu --graph rcu.dot
dot -Tsvg rcu.dot > rcu.svg

# Export JSON trace for analysis
vdlpp rcu --json trace.json
```

### View RCU Model Source

```bash
vdlpp show-rcu
```

### Parse a VDL++ Specification

```bash
vdlpp parse myspec.vdl
```

### Display Heritage Information

```bash
vdlpp info
```

## VDL++ Syntax

VDL++ uses a modern ML/TypeScript-inspired syntax while maintaining semantic rigor from the Vienna tradition.

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

VDL++ implements a **hybrid semantic approach**:

### Denotational Core
- Types define mathematical domains
- Expressions have denotational meaning
- Invariants are predicates over domains
- Functions are mathematical mappings

### Operational Layer
- Transitions are state transformers
- Pre/post conditions define valid transitions
- Small-step semantics for execution
- Trace generation for debugging

This hybrid approach is faithful to the meta-IV tradition where denotational foundations support operational rules.

## RCU Model

The flagship model demonstrates VDL++ capabilities by formalizing the Linux kernel's RCU (Read-Copy-Update) synchronization primitive:

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
┌─────────────────┐
│   VDL++ Source  │
└────────┬────────┘
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
    │     AST     │ (Denotational structure)
    └────┬────────┘
         │
    ┌────▼────────┐
    │  Evaluator  │ (Expression semantics)
    └────┬────────┘
         │
    ┌────▼────────────┐
    │  State Explorer │ (Operational semantics)
    └────┬────────────┘
         │
    ┌────▼──────────┐
    │   Outputs     │
    │ - Traces      │
    │ - Violations  │
    │ - DOT graphs  │
    │ - JSON        │
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

- **Enhanced parser** for full VDL++ syntax
- **VS Code extension** with Language Server Protocol (LSP)
- **Web IDE** with Monaco editor and WASM backend
- **Litmus test generation** compatible with LKMM
- **SRCU and Tasks RCU** variant models
- **Memory ordering constraints** integration
- **Symbolic execution** for deeper analysis
- **Property-based testing** generation from specs

## For Kernel Developers

VDL++ aims to provide:
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

VDL++ is a modern continuation of this mathematical tradition, bringing executable formal specifications to contemporary systems programming challenges.

## License

MIT License

---

*"For the Vienna Lab alumni: Executable semantics live again."*
