# VDL_2026 Example Library

Practical concurrent system specifications with tested implementations.

## Directory Structure

```
examples/
├── rcu/                    # Read-Copy-Update variations
│   ├── basic_rcu.vdl       # Simple RCU from main examples
│   ├── tree_rcu.vdl        # Hierarchical RCU
│   └── srcu.vdl            # Sleepable RCU
├── seqlock/                # Sequence locks
│   ├── basic_seqlock.vdl   # Standard seqlock
│   └── seqlock_writer.vdl  # Writer-biased variant
├── hazard_pointers/        # Memory reclamation
│   ├── hp_basic.vdl        # Basic hazard pointers
│   └── hp_multi.vdl        # Multi-reader hazard pointers
├── lockfree/               # Lock-free data structures
│   ├── queue.vdl           # Michael-Scott queue
│   ├── stack.vdl           # Treiber stack
│   └── counter.vdl         # Atomic counter
└── distributed/            # Distributed protocols
    ├── two_phase_commit.vdl # 2PC consensus
    ├── paxos_simple.vdl     # Simplified Paxos
    └── raft_leader.vdl      # Raft leader election
```

## Running Examples

```bash
# RCU exploration
cargo run -- rcu --depth 10 --readers 3

# Parse and check a specification
cargo run -- parse examples/seqlock/basic_seqlock.vdl

# Run tests
cargo test --all
```

## Examples by Category

### 1. RCU (Read-Copy-Update)

**basic_rcu.vdl**: Canonical RCU implementation
- Read-side critical sections
- Grace period management
- Callback execution
- Invariants: callback safety, epoch monotonicity

**tree_rcu.vdl**: Hierarchical RCU for multi-core scalability
- Per-CPU reader tracking
- Tree-structured grace period propagation
- Batch callback processing

**srcu.vdl**: Sleepable RCU variant
- Allows blocking in read-side sections
- Per-SRCU-struct synchronization
- Expedited grace periods

### 2. Seqlocks

**basic_seqlock.vdl**: Writer-optimized reader-writer lock
- Sequence number protocol
- Reader retry on write conflict
- No reader starvation

**seqlock_writer.vdl**: Writer-biased with write queueing
- Multiple concurrent writers
- FIFO writer queue
- Reader fast-path optimization

### 3. Hazard Pointers

**hp_basic.vdl**: Memory reclamation for lock-free structures
- Per-thread hazard pointer slots
- Safe memory reclamation
- ABA problem prevention

**hp_multi.vdl**: Multiple hazard pointers per thread
- K-hazard pointer slots
- Retirement list management
- Bounded reclamation

### 4. Lock-Free Data Structures

**queue.vdl**: Michael-Scott two-lock-free queue
- CAS-based enqueue/dequeue
- Linearizability
- Progress guarantees

**stack.vdl**: Treiber lock-free stack
- Single-word CAS operations
- ABA problem handling
- LIFO ordering

**counter.vdl**: Fetch-and-add counter
- Atomic increment/decrement
- Linearizable reads
- Overflow handling

### 5. Distributed Protocols

**two_phase_commit.vdl**: Atomic commit protocol
- Coordinator-participant model
- Prepare/commit phases
- Timeout and failure handling

**paxos_simple.vdl**: Consensus algorithm
- Proposer/acceptor/learner roles
- Majority quorum
- Safety and liveness properties

**raft_leader.vdl**: Leader election component
- Term-based election
- Randomized timeouts
- Split-vote prevention

## Testing

Each example includes:
- **Invariants**: Safety properties checked at every state
- **Test cases**: Specific scenarios to explore
- **Expected results**: Known good/bad states

Run tests with:
```bash
cargo test examples
```

## Contributing Examples

To add a new example:
1. Create `.vdl` specification file
2. Define clear invariants
3. Add test case in `tests/`
4. Document expected behavior
5. Update this README

## Citation

If you use these examples in research, please cite:

```bibtex
@software{vdl_2026,
  title = {VDL\_2026: IMPLICIT NONE for Concurrent Systems},
  year = {2026},
  url = {https://github.com/pacedproton/vdl_2026}
}
```
