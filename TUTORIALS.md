# VDL_2026 Tutorial Guide

**Learn VDL_2026 through practical examples with tested implementations**

---

## Quick Start

```bash
# 1. Build the project
cargo build

# 2. Run all tutorial tests
cargo test tutorials

# 3. Parse a tutorial specification
cargo run -- parse examples/tutorials/01_basic_counter.vdl

# 4. Explore the examples
cat examples/tutorials/README.md
```

## Tutorial Series

### 📚 Beginner Level

#### Tutorial 1: Basic Counter (⭐)

**File**: `examples/tutorials/01_basic_counter.vdl`

**Learn:**
- Basic VDL_2026 syntax
- State definitions with `type`
- Nat and Bool types
- Transitions with `pre` and `post` conditions

**Key Concepts:**
```vdl
type CounterState = {
  count: Nat,
  running: Bool
}

transition increment(s: CounterState) -> CounterState
pre s.running == true
post s'.count == s.count + 1 and
     s'.running == s.running
```

**Rust Model**: `src/models/tutorials/tutorial01_counter.rs`
- **13 tests** - all passing ✓
- Tests: preconditions, postconditions, complete sequences

**Run Tests:**
```bash
cargo test tutorial_01
```

---

#### Tutorial 2: Set Operations (⭐⭐)

**File**: `examples/tutorials/02_sets.vdl`

**Learn:**
- `Set<T>` type
- Set operations: `union`, `intersect`, `\` (difference)
- Set membership: `in`, `not_in`
- Quantifiers: `forall`, `exists`
- Set cardinality: `card(S)`

**Key Concepts:**
```vdl
type ProcessSet = {
  active: Set<Nat>,
  waiting: Set<Nat>,
  finished: Set<Nat>
}

inv processes_disjoint(s: ProcessSet) =
  s.active intersect s.waiting == {} and
  s.active intersect s.finished == {}

transition activate(s: ProcessSet, pid: Nat) -> ProcessSet
pre pid in s.waiting
post s'.active == s.active union {pid} and
     s'.waiting == s.waiting \ {pid}
```

**Demonstrates:**
- Process lifecycle (waiting → active → finished)
- Disjoint set invariants
- Quantified invariants

---

#### Tutorial 3: State Machine (⭐⭐)

**File**: `examples/tutorials/03_state_machine.vdl`

**Learn:**
- Enumerated state modeling (using Nat encoding)
- State transition tables
- Guard conditions
- Conditional transitions with `if-then-else`

**Key Concepts:**
```vdl
type ConnectionState = {
  state: Nat,        // 0=Idle, 1=Connecting, 2=Connected, ...
  retry_count: Nat,
  timeout: Nat
}

transition connection_failed(s: ConnectionState) -> ConnectionState
pre s.state == 1
post (if s.retry_count < 3
      then s'.state == 1 and s'.retry_count == s.retry_count + 1
      else s'.state == 4)  // Move to Error state
```

**Demonstrates:**
- Connection protocol with retries
- Timeout handling
- Error states and recovery

---

### 🎯 Intermediate Level

#### Tutorial 4: Bounded Resource (⭐⭐⭐)

**File**: `examples/tutorials/04_bounded_resource.vdl`

**Learn:**
- Type invariants (in type definition with `where`)
- State invariants (`inv`)
- Invariant violation detection
- Resource management

**Key Concepts:**
```vdl
type ResourcePool = {
  allocated: Nat,
  available: Nat,
  total: Nat,
  requests: Nat
}

inv total_correct(s: ResourcePool) =
  s.total == s.allocated + s.available

inv capacity_bounded(s: ResourcePool) =
  s.total <= 100

transition allocate(s: ResourcePool, amount: Nat) -> ResourcePool
pre amount > 0 and amount <= s.available
post s'.allocated == s.allocated + amount and
     s'.available == s.available - amount
```

**Rust Model**: `src/models/tutorials/tutorial04_bounded_resource.rs`
- **11 tests** - all passing ✓
- Tests: invariant checking, boundary conditions, complete scenarios

**Demonstrates:**
- Resource allocation/release
- Capacity expansion/shrinking
- Invariant preservation
- Precondition enforcement preventing violations

**Run Tests:**
```bash
cargo test tutorial_04
```

---

#### Tutorial 5: Key-Value Store (⭐⭐⭐)

**File**: `examples/tutorials/05_key_value_store.vdl`

**Learn:**
- `Map<K, V>` type
- Map operations: `dom`, `rng`, `with`, `munion`
- Map application: `map(key)`
- Record field updates
- CRUD operations

**Key Concepts:**
```vdl
type KVStore = {
  data: Map<Key, Value>,
  size: Nat,
  capacity: Nat,
  operations: Nat
}

inv size_matches_map(s: KVStore) =
  s.size == card(dom(s.data))

transition put(s: KVStore, key: Key, value: Value) -> KVStore
pre (key not_in dom(s.data) implies s.size < s.capacity)
post s'.data == s.data with [key |-> value] and
     s'.size == (if key in dom(s.data) then s.size else s.size + 1)
```

**Demonstrates:**
- Insert/update distinction
- Delete operations
- Bulk operations
- Map domain/range queries

---

### 🚀 Advanced Level

#### Tutorial 6: Mutual Exclusion (⭐⭐⭐⭐)

**File**: `examples/tutorials/06_mutex.vdl`

**Learn:**
- Multi-process coordination
- Lock acquisition protocol
- Waiting queues
- Critical section protection
- Complex invariants

**Key Concepts:**
```vdl
type MutexState = {
  locked: Bool,
  holder: Nat,
  waiting: Set<ProcessId>,
  in_critical: Set<ProcessId>
}

inv mutual_exclusion(s: MutexState) =
  card(s.in_critical) <= 1

inv lock_consistency(s: MutexState) =
  s.locked iff (s.holder != 0 and card(s.in_critical) == 1)

transition lock_request(s: MutexState, pid: ProcessId) -> MutexState
pre pid > 0 and pid not_in s.waiting and pid not_in s.in_critical
post (if s.locked
      then s'.waiting == s.waiting union {pid}
      else s'.holder == pid and s'.in_critical == {pid})
```

**Rust Model**: `src/models/tutorials/tutorial06_mutex.rs`
- **13 tests** - all passing ✓
- Tests: mutual exclusion, multi-process scenarios, invariant checking

**Demonstrates:**
- Lock/unlock protocol
- Process waiting queue
- Mutual exclusion guarantee
- Three-process scenario
- Impossibility of violations by construction

**Run Tests:**
```bash
cargo test tutorial_06
```

---

## Test Summary

```bash
cargo test tutorials --lib
```

**Results:**
```
running 37 tests

Tutorial 01 (Counter):          13 tests ✓
Tutorial 04 (Resource):         11 tests ✓
Tutorial 06 (Mutex):            13 tests ✓

test result: ok. 37 passed; 0 failed
```

### Test Coverage

Each tutorial tests:
- ✅ Initial states
- ✅ Valid transitions
- ✅ Blocked transitions (precondition enforcement)
- ✅ Postcondition correctness
- ✅ Complete execution scenarios
- ✅ Invariant preservation
- ✅ Invariant violation detection (Tutorial 4)
- ✅ Multi-process scenarios (Tutorial 6)

---

## Learning Path

### Step 1: Master Basics (Tutorials 1-3)

Start here if you're new to VDL_2026 or formal methods:

1. **Tutorial 1** - Understand syntax, types, transitions
2. **Tutorial 2** - Learn set operations and quantifiers
3. **Tutorial 3** - Model state machines

**Time**: 1-2 hours

### Step 2: Understand Invariants (Tutorial 4)

Critical for verification:

4. **Tutorial 4** - See how invariants prevent bugs

**Key Insight**: Well-designed invariants catch errors before they propagate!

**Time**: 30 minutes

### Step 3: Complex Data (Tutorial 5)

Real systems need maps and records:

5. **Tutorial 5** - Master map operations

**Time**: 45 minutes

### Step 4: Concurrent Protocols (Tutorial 6)

The ultimate goal:

6. **Tutorial 6** - Understand mutual exclusion

**Challenge**: Try to violate mutual exclusion. You can't!

**Time**: 1 hour

**Total Learning Time**: ~4 hours to proficiency

---

## Using Tutorials with State Explorer

While full state exploration integration is in progress, you can already:

### 1. Parse Specifications

```bash
cargo run -- parse examples/tutorials/04_bounded_resource.vdl
```

Verifies syntax and type definitions.

### 2. Run Rust Models

```bash
cargo test tutorial_04 -- --nocapture
```

Shows execution traces in test output.

### 3. Check Invariants

```rust
use vdl_2026::models::tutorials::tutorial04_bounded_resource::*;

let pool = ResourcePool::new(50);
assert!(pool.check_all_invariants());

// After allocation
let s1 = apply_resource_transition(&make_resource_pool(0, 50, 50, 0), "allocate", 30);
let violations = check_invariants(&s1.unwrap());
assert!(violations.is_empty());  // No violations!
```

---

## Common Patterns

### Pattern 1: Bounded Resources

**Problem**: Prevent resource exhaustion

**Solution**: (Tutorial 4)
```vdl
inv capacity_bounded(s: ResourcePool) =
  s.total <= 100

transition expand(s: ResourcePool, amount: Nat) -> ResourcePool
pre s.total + amount <= 100  // Enforces invariant
```

### Pattern 2: Disjoint Sets

**Problem**: Entities should belong to only one category

**Solution**: (Tutorial 2)
```vdl
inv processes_disjoint(s: ProcessSet) =
  s.active intersect s.waiting == {} and
  s.active intersect s.finished == {}
```

### Pattern 3: Conditional Transitions

**Problem**: Different behavior based on state

**Solution**: (Tutorial 3, 6)
```vdl
post (if condition
      then state_a_updates
      else state_b_updates)
```

### Pattern 4: Mutual Exclusion

**Problem**: At most N entities can access resource

**Solution**: (Tutorial 6)
```vdl
inv mutual_exclusion(s: MutexState) =
  card(s.in_critical) <= 1

transition lock_request(s: MutexState, pid: Nat) -> MutexState
pre pid not_in s.in_critical
post (if s.locked then ... else s'.in_critical == {pid})
```

---

## Extending Tutorials

### Challenge Exercises

After completing tutorials, try:

**Tutorial 1 Extensions:**
- Add `max_value` field and bounded increment
- Add `step_size` for variable increments
- Implement `reset_to(value)` transition

**Tutorial 2 Extensions:**
- Add priority levels for processes
- Implement round-robin scheduling
- Add `migrate(pid, from_set, to_set)` transition

**Tutorial 4 Extensions:**
- Add process IDs tracking allocations
- Implement fair allocation (FIFO queue)
- Add `reserve` and `commit` two-phase allocation

**Tutorial 6 Extensions:**
- Add timeout for lock acquisition
- Implement priority-based lock assignment
- Model recursive locks (same process locks multiple times)

### Creating New Tutorials

Follow the template:

1. **VDL Specification** - Clear comments explaining concepts
2. **Rust Model** - State struct, transitions, invariants
3. **Tests** - Comprehensive coverage
4. **Documentation** - Learning objectives, expected behavior

See `examples/tutorials/README.md` for guidelines.

---

## FAQ

**Q: Why are some tutorials missing Rust models?**

A: Tutorials 1, 4, and 6 have full Rust implementations to demonstrate the range of complexity. You can implement the others as exercises!

**Q: Can I run the VDL specifications directly?**

A: The VDL interpreter is in development. Currently, parse with `cargo run -- parse <file.vdl>` and run Rust models with `cargo test`.

**Q: How do invariants actually prevent bugs?**

A: See Tutorial 4's `test_invariant_checking` test. Invalid states are detected immediately, not after they cause problems!

**Q: What's next after tutorials?**

A: Explore real-world examples:
- `examples/seqlock/` - Writer-optimized synchronization
- `examples/lockfree/stack.vdl` - Lock-free data structures
- `examples/distributed/two_phase_commit.vdl` - Consensus protocols

---

## References

- **Language Spec**: `docs/VDL_2026_PLUS_SPECIFICATION.md`
- **Research**: `docs/RESEARCH_VDL_2026_PLUS.md`
- **Examples**: `examples/README.md`
- **Status**: `STATUS.md`

---

## Contributing

Have ideas for new tutorials? Found a bug? Want to add Rust models for tutorials 2, 3, 5?

1. Follow existing tutorial format
2. Include comprehensive tests
3. Update this guide
4. Submit PR!

---

**Happy Learning! 🎓**

*VDL_2026: IMPLICIT NONE for concurrent systems*
