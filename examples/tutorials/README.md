# VDL_2026 Tutorial Examples

Progressive tutorial series demonstrating VDL_2026 features with tested implementations.

## Tutorial Index

### Beginner Level

**Tutorial 1: Basic Counter** (`01_basic_counter.vdl`)
- Learn: Basic types (Nat, Bool), simple transitions
- Concepts: State definition, pre/post conditions
- Complexity: ⭐

**Tutorial 2: Set Operations** (`02_sets.vdl`)
- Learn: Set types, quantifiers (forall, exists)
- Concepts: Set union/intersect/difference, membership
- Complexity: ⭐⭐

**Tutorial 3: State Machine** (`03_state_machine.vdl`)
- Learn: Enumerated states, guarded transitions
- Concepts: State transitions, guards
- Complexity: ⭐⭐

### Intermediate Level

**Tutorial 4: Bounded Resource** (`04_bounded_resource.vdl`)
- Learn: Invariants, violation detection
- Concepts: Type invariants, state invariants
- Complexity: ⭐⭐⭐

**Tutorial 5: Key-Value Store** (`05_key_value_store.vdl`)
- Learn: Maps, records, complex updates
- Concepts: Map operations, record fields
- Complexity: ⭐⭐⭐

**Tutorial 6: Mutex Protocol** (`06_mutex.vdl`)
- Learn: Mutual exclusion, multi-process interaction
- Concepts: Process IDs, mutual exclusion invariants
- Complexity: ⭐⭐⭐⭐

### Advanced Level

**Tutorial 7: Producer-Consumer** (`07_producer_consumer.vdl`)
- Learn: Queues, bounded buffers
- Concepts: FIFO ordering, capacity constraints
- Complexity: ⭐⭐⭐⭐

**Tutorial 8: Leader Election** (`08_leader_election.vdl`)
- Learn: Distributed coordination
- Concepts: Majority quorums, epochs
- Complexity: ⭐⭐⭐⭐⭐

## Running Tutorials

### Parse and Validate

```bash
cargo run -- parse examples/tutorials/01_basic_counter.vdl
```

### Explore State Space

Each tutorial includes a Rust model in `src/models/tutorials/`:

```bash
cargo test tutorial_01  # Run specific tutorial tests
cargo test tutorials    # Run all tutorial tests
```

### Interactive Exploration

```bash
# Tutorial 1: Counter
cargo run -- tutorial 01

# Tutorial 4: Bounded resource
cargo run -- tutorial 04 --max-capacity 10
```

## Learning Path

1. **Start with Tutorial 1-3**: Learn basic syntax and transitions
2. **Move to Tutorial 4-6**: Understand invariants and protocols
3. **Challenge with Tutorial 7-8**: Master complex concurrent systems

## Tutorial Format

Each tutorial includes:

1. **VDL Specification** (`.vdl` file)
   - Clear learning objectives
   - Inline comments explaining concepts
   - Invariants and properties

2. **Rust Model** (`src/models/tutorials/tutorialNN.rs`)
   - State representation
   - Transition functions
   - Invariant checkers

3. **Tests** (`tests/tutorials/tutorialNN_test.rs`)
   - Unit tests for transitions
   - Integration tests with state explorer
   - Expected behavior validation

4. **README** (this file)
   - Concepts covered
   - Learning objectives
   - Expected output

## Prerequisites

- Basic understanding of formal methods concepts
- Familiarity with sets, maps, and logic
- Rust basics (for reading implementation)

## Tips

- Read VDL spec first, understand the protocol
- Run tests to see expected behavior
- Modify examples and re-run tests to experiment
- Check `STATUS.md` for language features in development

## Next Steps

After completing tutorials:
- Explore `examples/seqlock/` for real-world concurrency
- Read `docs/SPECIFICATION.md` for complete language reference
- Try modeling your own protocols!

## Contributing

Found an issue or want to add a tutorial?
- Open issue on GitHub
- PRs welcome for new tutorials
- Follow existing tutorial format
