# VDL_2026+ Language Specification

**Vienna Definition Language 2026 Plus**

**IMPLICIT NONE for Concurrent Systems with Temporal Verification**

Version 0.2.0

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Lexical Structure](#2-lexical-structure)
3. [Type System](#3-type-system)
4. [Expressions](#4-expressions)
5. [Transitions](#5-transitions)
6. [Process Algebra](#6-process-algebra)
7. [Temporal Logic](#7-temporal-logic)
8. [Dependent Types](#8-dependent-types)
9. [Refinement Types](#9-refinement-types)
10. [Invariants and Properties](#10-invariants-and-properties)
11. [Module System](#11-module-system)
12. [Proof Automation](#12-proof-automation)
13. [Execution Semantics](#13-execution-semantics)
14. [Formal Semantics](#14-formal-semantics)
15. [Examples](#15-examples)

---

## 1. Introduction

### 1.1 Overview

VDL_2026+ extends VDL_2026 with:
- **Process algebra** (CCS/CSP-inspired) for concurrent composition
- **Temporal logic** (LTL/CTL) for safety and liveness properties
- **Dependent types** for rich specifications
- **Refinement types** for automatic verification
- **Proof automation** with SMT integration

### 1.2 Design Principles

1. **Surpass by synthesis**: Combine strengths of CCS, CSP, Z, TLA+, Lean/Coq
2. **Practical syntax**: ASCII-friendly, no mathematical symbols required
3. **Executable specifications**: Run, simulate, and verify
4. **Integrated tooling**: VSCode LSP, model checker, proof search
5. **Vienna tradition**: mk_, is_, inv_ conventions with modern ergonomics

### 1.3 Comparison with Predecessors

| Feature | VDL_2026 | VDL_2026+ |
|---------|----------|-----------|
| Process algebra | ✗ | ✓ |
| Temporal logic | Basic | LTL/CTL |
| Dependent types | ✗ | ✓ |
| Refinement types | ✗ | ✓ |
| SMT integration | ✗ | ✓ |
| Proof automation | ✗ | ✓ |

---

## 2. Lexical Structure

### 2.1 Keywords

**Type keywords:**
```
type      -- Type definition
where     -- Type constraint
requires  -- Precondition
ensures   -- Postcondition
```

**Process algebra keywords:**
```
process   -- Process definition
parallel  -- Parallel composition (|)
interleave -- Interleaved composition (|||)
sequence  -- Sequential composition (->)
choice    -- Non-deterministic choice (+)
```

**Temporal keywords:**
```
temporal  -- Temporal property
always    -- □ (box, globally)
eventually -- ◇ (diamond, finally)
next      -- X (next state)
until     -- U (strong until)
release   -- R (release operator)
```

**Proof keywords:**
```
proof     -- Proof block
lemma     -- Lemma definition
theorem   -- Theorem statement
by        -- Proof tactic
auto      -- Automatic proof search
smt       -- SMT solver
induction -- Induction tactic
unfold    -- Unfold definitions
model_check -- Model checking tactic
```

**Module keywords:**
```
module    -- Module definition
export    -- Export declaration
import    -- Import declaration
```

### 2.2 Operators

**Temporal operators (ASCII-friendly):**
```
[]        -- always (globally)
<>        -- eventually (finally)
~>        -- leads to (p ~> q ≡ [](p => <>q))
```

**Process operators:**
```
|         -- Parallel composition
|||       -- Interleaving
->        -- Sequential transition
+         -- Non-deterministic choice
\         -- Restriction (hiding)
```

**Logical operators:**
```
and, or, not, implies, iff
forall, exists
```

**Type operators:**
```
->        -- Function type (in type context)
=>        -- Dependent function type
|         -- Refinement (in type context)
```

### 2.3 Reserved Symbols

```
@         -- Temporal reference (@next, @prev)
#         -- Cardinality (legacy: card)
::        -- Module path separator
<:        -- Subtype relation
```

---

## 3. Type System

### 3.1 Base Types

```
Nat       -- Natural numbers (0, 1, 2, ...)
Int       -- Integers (..., -1, 0, 1, ...)
Bool      -- Booleans (true, false)
Unit      -- Unit type (⊤)
```

### 3.2 Collection Types

```
Set<T>            -- Finite set
List<T>           -- Sequence/list
Map<K, V>         -- Finite map (partial function)
```

### 3.3 Composite Types

```
type Point = {
  x: Int,
  y: Int
}

type Option<T> =
  | Some(T)
  | None

type Result<T, E> =
  | Ok(T)
  | Err(E)
```

### 3.4 Dependent Types

```
-- Length-indexed vectors
type Vec<T>(n: Nat) = {
  data: List<T>
} where len(data) == n

-- Bounded integers
type BoundedInt(lo: Int, hi: Int) = {
  val: Int
} where lo <= val and val <= hi

-- Non-empty lists
type NonEmpty<T> = {
  elems: List<T>
} where len(elems) > 0
```

### 3.5 Refinement Types

```
-- Subset types with predicates
type Even = { n: Nat | n % 2 == 0 }
type Odd = { n: Nat | n % 2 == 1 }
type NonZero = { x: Int | x != 0 }
type Positive = { x: Int | x > 0 }

-- State-dependent refinement
type ValidReader(s: RcuState) = { r: Nat | r in s.readers }
type PendingCallback(s: RcuState) = { cb: Callback | cb in s.pending }
```

### 3.6 Function Types

```
-- Simple function
fun add(x: Int, y: Int) -> Int

-- Dependent function type
fun get<T>(v: Vec<T>(n), i: Nat) -> T
  requires i < n

-- Refinement-preserving function
fun double(n: Even) -> Even

-- State transformer
transition increment(s: State, x: Id) -> State
  ensures s'.vars(x) == s.vars(x) + 1
```

---

## 4. Expressions

### 4.1 Literals

```
42                -- Nat literal
-17               -- Int literal
true, false       -- Bool literals
{}                -- Empty set
[]                -- Empty list
{x |-> v}         -- Singleton map
```

### 4.2 Collection Expressions

```
{1, 2, 3}                    -- Set literal
[1, 2, 3]                    -- List literal
{x |-> 1, y |-> 2}           -- Map literal
{x in S | P(x)}              -- Set comprehension
[f(x) | x in xs]             -- List comprehension
{k |-> f(k) | k in ks}       -- Map comprehension
```

### 4.3 Operators

```
-- Arithmetic
+, -, *, /, %

-- Comparison
==, !=, <, <=, >, >=

-- Logical
and, or, not, implies, iff

-- Set
union, intersect, \, in, not_in, subset_of
card(s), {x in s | P(x)}

-- List
++, head, tail, len, elems, (i)

-- Map
dom, rng, with, munion, (k)
```

### 4.4 Temporal References

```
@next(e)          -- Value of e in next state (e')
@prev(e)          -- Value of e in previous state
@stable(e)        -- e is stable (e == @next(e))
```

---

## 5. Transitions

### 5.1 Basic Transitions

```
transition rcu_read_lock(s: RcuState, r: Nat) -> RcuState
pre r not_in s.readers
post s'.readers == s.readers union {r} and
     s'.epoch == s.epoch and
     s'.pending == s.pending
```

### 5.2 Guarded Transitions

```
transition timeout(s: State) -> State
guard s.elapsed > TIMEOUT_MS
post s'.timed_out == true and
     s'.elapsed == 0
```

### 5.3 Non-Deterministic Transitions

```
transition choose_action(s: State) -> State
choice {
  case action_a:
    post s'.action == Action_A
  case action_b:
    post s'.action == Action_B
  case action_c:
    post s'.action == Action_C
}
```

### 5.4 Conditional Transitions

```
transition process_request(s: State, req: Request) -> State
if req.priority == High then
  post s'.queue == [req] ++ s.queue
else
  post s'.queue == s.queue ++ [req]
```

---

## 6. Process Algebra

### 6.1 Process Definitions

```
-- Process type
process Reader(id: Nat, s: RcuState) : Process

-- Process with transitions
process Reader(id: Nat, s: RcuState) =
  rcu_read_lock(s, id) -> Reader_Critical(id, s)

process Reader_Critical(id: Nat, s: RcuState) =
  access_data(s, id) ->
  rcu_read_unlock(s, id) ->
  Reader(id, s)
```

### 6.2 Parallel Composition

```
-- Synchronous parallel (CCS-style)
process System(s: RcuState) =
  Reader(1, s) | Writer(s)

-- Interleaved parallel (CSP-style)
process System(s: RcuState) =
  Reader(1, s) ||| Reader(2, s) ||| Writer(s)

-- Generalized parallel
process System(s: RcuState) =
  parallel(i in 1..NUM_READERS) { Reader(i, s) }
```

### 6.3 Sequential Composition

```
process Protocol(s: State) =
  prepare(s) ->
  commit(s) ->
  acknowledge(s) ->
  Protocol(s')
```

### 6.4 Choice Operators

```
-- External choice (environment decides)
process Client(s: State) =
  send_request(s) + receive_response(s)

-- Internal choice (system decides)
process Server(s: State) =
  accept_connection(s) ⊓ reject_connection(s)
```

### 6.5 Restriction

```
-- Hide internal transitions
process System(s: State) =
  (Reader(s) | Writer(s)) \ {internal_sync}
```

---

## 7. Temporal Logic

### 7.1 LTL (Linear Temporal Logic)

```
-- Basic operators
temporal always(p: State -> Bool) -> LTL        -- □p
temporal eventually(p: State -> Bool) -> LTL    -- ◇p
temporal next(p: State -> Bool) -> LTL          -- Xp
temporal until(p: State -> Bool, q: State -> Bool) -> LTL  -- p U q

-- Derived operators
temporal leads_to(p: State -> Bool, q: State -> Bool) -> LTL =
  always(p implies eventually(q))               -- □(p ⇒ ◇q)

temporal infinitely_often(p: State -> Bool) -> LTL =
  always(eventually(p))                         -- □◇p

temporal eventually_always(p: State -> Bool) -> LTL =
  eventually(always(p))                         -- ◇□p
```

### 7.2 CTL (Computation Tree Logic)

```
-- Path quantifiers + temporal operators
ctl exists_next(p: State -> Bool) -> CTL        -- EX p
ctl exists_eventually(p: State -> Bool) -> CTL  -- EF p
ctl exists_always(p: State -> Bool) -> CTL      -- EG p
ctl exists_until(p: State -> Bool, q: State -> Bool) -> CTL  -- E[p U q]

ctl forall_next(p: State -> Bool) -> CTL        -- AX p
ctl forall_eventually(p: State -> Bool) -> CTL  -- AF p
ctl forall_always(p: State -> Bool) -> CTL      -- AG p
ctl forall_until(p: State -> Bool, q: State -> Bool) -> CTL  -- A[p U q]
```

### 7.3 Past-Time LTL

```
temporal previous(p: State -> Bool) -> PTLTL    -- Y p (yesterday)
temporal once(p: State -> Bool) -> PTLTL        -- ⊟ p (once in past)
temporal historically(p: State -> Bool) -> PTLTL -- ⊞ p (always in past)
temporal since(p: State -> Bool, q: State -> Bool) -> PTLTL  -- p S q
```

### 7.4 Temporal Properties

```
property safety_mutual_exclusion =
  always(not (reader1_critical and reader2_critical))

property liveness_eventual_callback =
  always(callback_queued implies eventually(callback_executed))

property fairness_starvation_free =
  always(request_pending(id) implies eventually(request_served(id)))

property response_time =
  always(request_sent implies eventually_within(response_received, 1000))
```

---

## 8. Dependent Types

### 8.1 Index Types

```
-- Type parameter depends on value parameter
type Matrix<T>(rows: Nat, cols: Nat) = {
  data: Vec<Vec<T>(cols)>(rows)
}

-- Dependent record
type BufferedChannel<T>(capacity: Nat) = {
  buffer: Vec<T>(capacity),
  head: BoundedInt(0, capacity),
  tail: BoundedInt(0, capacity),
  count: BoundedInt(0, capacity)
} where
  (tail + count) % capacity == head
```

### 8.2 Dependent Functions

```
-- Length-preserving map
fun map<A, B>(f: A -> B, xs: Vec<A>(n)) -> Vec<B>(n) =
  mk_Vec([f(x) | x in xs.data], n)

-- Safe indexing
fun nth<T>(xs: Vec<T>(n), i: Nat) -> T
  requires i < n =
  xs.data(i)

-- Safe division
fun divide(x: Int, y: NonZero) -> Int =
  x / y.val
```

### 8.3 Dependent Pairs (Σ-types)

```
-- Existential pair: (value, proof)
type Sigma<A, P: A -> Type> = {
  fst: A,
  snd: P(fst)
}

-- Example: witness with proof
type Witness<P: Nat -> Bool> = {
  n: Nat,
  proof: P(n) == true
}
```

---

## 9. Refinement Types

### 9.1 Basic Refinements

```
type Even = { n: Nat | n % 2 == 0 }
type Odd = { n: Nat | n % 2 == 1 }
type Positive = { x: Int | x > 0 }
type Negative = { x: Int | x < 0 }
type NonZero = { x: Int | x != 0 }

-- Range types
type Percentage = { p: Int | 0 <= p and p <= 100 }
type Byte = { b: Nat | b < 256 }
```

### 9.2 State-Dependent Refinements

```
-- Reader must be in readers set
type ActiveReader(s: RcuState) = { r: Nat | r in s.readers }

-- Callback must be pending
type PendingCallback(s: RcuState) = { cb: Callback | cb in s.pending }

-- Valid array index
type ValidIndex<T>(xs: List<T>) = { i: Nat | i < len(xs) }
```

### 9.3 Refinement-Preserving Functions

```
-- Input/output refinements automatically checked
fun double_even(n: Even) -> Even =
  n * 2  -- Type checker proves: (n % 2 == 0) ⇒ ((n * 2) % 2 == 0)

fun add_positive(x: Positive, y: Positive) -> Positive =
  x + y  -- Proves: (x > 0 ∧ y > 0) ⇒ (x + y > 0)

fun safe_div(x: Int, y: NonZero) -> Int =
  x / y  -- Proves: y != 0 (no division by zero)
```

### 9.4 Refinement Subtyping

```
-- Subtype relation
Even <: Nat              -- Every Even is a Nat
NonZero <: Int           -- Every NonZero is an Int
Vec<T>(n) <: List<T>     -- Every Vec<T>(n) is a List<T>

-- Covariance
f: Even -> Nat
g: Nat -> Nat
g <: f                   -- f accepts more inputs

-- Contravariance
h: Nat -> Even
k: Nat -> Nat
h <: k                   -- h produces more specific outputs
```

---

## 10. Invariants and Properties

### 10.1 Type Invariants

```
type RcuState = {
  readers: Set<Nat>,
  epoch: Nat,
  pending: List<Callback>
} where
  card(readers) <= MAX_READERS and
  epoch > 0 and
  forall cb in pending: cb.epoch <= epoch
```

### 10.2 State Invariants

```
inv callback_safety(s: RcuState) =
  forall cb in s.pending:
    cb.epoch <= s.epoch

inv no_reader_overflow(s: RcuState) =
  card(s.readers) <= MAX_READERS

inv epoch_positive(s: RcuState) =
  s.epoch > 0
```

### 10.3 Transition Invariants

```
-- Invariant preserved by transition
transition rcu_read_lock(s: RcuState, r: Nat) -> RcuState
pre r not_in s.readers
post s'.readers == s.readers union {r}
preserves callback_safety, epoch_positive
```

### 10.4 Safety Properties

```
-- Something bad never happens
safety mutual_exclusion =
  always(forall r1, r2 in state.readers:
    r1 != r2 implies not (critical(r1) and critical(r2)))

safety no_buffer_overflow =
  always(card(state.buffer) <= CAPACITY)
```

### 10.5 Liveness Properties

```
-- Something good eventually happens
liveness eventual_callback_completion =
  always(exists cb in state.pending implies
    eventually(cb not_in state.pending))

liveness starvation_freedom =
  always(forall req in state.queue:
    eventually(processed(req)))
```

### 10.6 Fairness Properties

```
-- Weak fairness: enabled infinitely often ⇒ executed infinitely often
fairness weak_fair(t: Transition) =
  infinitely_often(enabled(t)) implies infinitely_often(executed(t))

-- Strong fairness: enabled continuously ⇒ executed infinitely often
fairness strong_fair(t: Transition) =
  eventually_always(enabled(t)) implies infinitely_often(executed(t))
```

---

## 11. Module System

### 11.1 Module Definition

```
module RcuCore {
  // Type exports
  export type RcuState
  export type Callback

  // Transition exports
  export transition rcu_read_lock
  export transition rcu_read_unlock

  // Invariant exports
  export inv callback_safety
  export inv epoch_positive

  // Implementation (private by default)
  type RcuState = {
    readers: Set<Nat>,
    epoch: Nat,
    pending: List<Callback>
  }

  transition rcu_read_lock(s: RcuState, r: Nat) -> RcuState
  pre r not_in s.readers
  post s'.readers == s.readers union {r}
}
```

### 11.2 Module Import

```
module RcuGracePeriod {
  import RcuCore.*  // Import all exports
  import RcuCore::{RcuState, rcu_read_lock}  // Selective import

  export transition start_grace_period
  export transition end_grace_period

  // Use imported types
  transition start_grace_period(s: RcuState) -> RcuState
  pre card(s.readers) == 0
  post s'.epoch == s.epoch + 1
}
```

### 11.3 Module Composition

```
module RcuSystem {
  import RcuCore
  import RcuGracePeriod

  // Composite system
  export process System

  process System(s: RcuCore::RcuState) =
    (Reader(1, s) | Reader(2, s)) ||| RcuGracePeriod::GracePeriodManager(s)

  // Inherited invariants
  requires RcuCore::callback_safety
  requires RcuCore::epoch_positive
  ensures RcuGracePeriod::grace_period_complete
}
```

### 11.4 Module Contracts

```
module Stack<T> {
  export type Stack<T>
  export fun push
  export fun pop

  // Interface contract
  requires true  // No preconditions on module use

  ensures forall s: Stack<T>, x: T:
    pop(push(s, x)) == Some(x, s)

  ensures forall s: Stack<T>:
    is_empty(s) implies pop(s) == None
}
```

---

## 12. Proof Automation

### 12.1 Automatic Tactics

```
proof double_even_correct:
  forall n: Even: is_even(double_even(n))
  by auto  // Automatic proof search

proof division_safe:
  forall x: Int, y: NonZero: x / y is defined
  by smt   // SMT solver (Z3, CVC5)
```

### 12.2 Induction

```
proof factorial_positive:
  forall n: Nat: factorial(n) > 0
  by induction on n {
    case 0:
      by arithmetic
    case n + 1:
      assume ih: factorial(n) > 0
      show factorial(n + 1) > 0
      by {
        unfold factorial;
        by arithmetic using ih
      }
  }
```

### 12.3 Model Checking Proofs

```
proof callback_safety_preserved:
  forall s: RcuState, r: Nat:
    callback_safety(s) implies callback_safety(rcu_read_lock(s, r))
  by model_check depth=10

proof mutual_exclusion_holds:
  always(not (reader1_critical and reader2_critical))
  by model_check depth=100 fairness=weak
```

### 12.4 Lemmas

```
lemma set_union_comm<T>:
  forall s1, s2: Set<T>:
    s1 union s2 == s2 union s1
  by auto

lemma list_append_assoc<T>:
  forall xs, ys, zs: List<T>:
    (xs ++ ys) ++ zs == xs ++ (ys ++ zs)
  by auto
```

### 12.5 Theorem Statements

```
theorem rcu_correctness:
  forall sys: RcuSystem:
    (always(callback_safety) and
     always(epoch_positive) and
     eventually(forall cb: cb not_in pending))
  by {
    split;
    by model_check;
    by model_check;
    by liveness_analysis
  }
```

---

## 13. Execution Semantics

### 13.1 Deterministic Execution

```
fun eval_expr(e: Expr, env: Env) -> Value =
  match e {
    Lit(n) => Value::Nat(n),
    Var(x) => env.lookup(x),
    BinOp(op, e1, e2) => apply_op(op, eval_expr(e1, env), eval_expr(e2, env))
  }
```

### 13.2 Non-Deterministic Execution

```
// State exploration explores all non-deterministic branches
transition choose(s: State) -> State
choice {
  case a: post s'.choice == A
  case b: post s'.choice == B
}

// Explorer generates: {(s, s[choice=A]), (s, s[choice=B])}
```

### 13.3 Process Interleaving

```
// Parallel composition: interleave all transitions
process P | Q generates interleavings:
  - Execute transition from P
  - Execute transition from Q
  - Execute synchronized transition from P and Q (if compatible)
```

### 13.4 Trace Semantics

```
// Trace: sequence of states
trace = [s0, s1, s2, ..., sn]

// Trace satisfies LTL formula
trace |= always(p) iff forall i: trace[i] |= p
trace |= eventually(p) iff exists i: trace[i] |= p
trace |= p until q iff exists j: (trace[j] |= q and forall i < j: trace[i] |= p)
```

---

## 14. Formal Semantics

### 14.1 Denotational Semantics

```
⟦_⟧: Expr → Env → Value

⟦n⟧ρ = n
⟦x⟧ρ = ρ(x)
⟦e₁ + e₂⟧ρ = ⟦e₁⟧ρ + ⟦e₂⟧ρ
⟦{x in S | P(x)}⟧ρ = {⟦x⟧ρ | x ∈ ⟦S⟧ρ ∧ ⟦P(x)⟧ρ = true}
```

### 14.2 Operational Semantics

```
-- Small-step transition relation
(s, t) → s'

-- Transition rule
─────────────────────────────────────
pre(s, t) ∧ post(s, s', t)
─────────────────────────────────────
(s, t) → s'

-- Parallel composition
(s, t₁) → s₁    (s, t₂) → s₂
─────────────────────────────
(s, t₁ | t₂) → {s₁, s₂}
```

### 14.3 Temporal Semantics

```
-- LTL satisfaction
π, i |= p iff π[i] ∈ ⟦p⟧
π, i |= □φ iff ∀j ≥ i: π, j |= φ
π, i |= ◇φ iff ∃j ≥ i: π, j |= φ
π, i |= φ U ψ iff ∃j ≥ i: (π, j |= ψ ∧ ∀k ∈ [i,j): π, k |= φ)

-- CTL satisfaction
s |= EX φ iff ∃s': s → s' ∧ s' |= φ
s |= EF φ iff ∃π starting at s, ∃i: π[i] |= φ
s |= EG φ iff ∃π starting at s, ∀i: π[i] |= φ
```

### 14.4 Refinement Semantics

```
-- Refinement type checking
Γ ⊢ e : {x: T | φ} iff Γ ⊢ e : T ∧ Γ ⊢ φ[e/x]

-- Subtyping
{x: T | φ₁} <: {x: T | φ₂} iff ∀x: φ₁(x) ⇒ φ₂(x)

-- Dependent function
Γ ⊢ f : (x: A) => B(x) iff
  ∀a: A, Γ ⊢ f(a) : B(a)
```

---

## 15. Examples

### 15.1 RCU with Temporal Properties

```
module RcuTemporal {
  type RcuState = {
    readers: Set<Nat>,
    epoch: Nat,
    pending: List<Callback>
  }

  transition rcu_read_lock(s: RcuState, r: Nat) -> RcuState
  pre r not_in s.readers
  post s'.readers == s.readers union {r}

  transition start_grace_period(s: RcuState) -> RcuState
  pre card(s.readers) == 0
  post s'.epoch == s.epoch + 1

  // Temporal properties
  property safety_callback_epoch =
    always(forall cb in state.pending: cb.epoch <= state.epoch)

  property liveness_callback_completion =
    always(card(state.pending) > 0 implies
      eventually(card(state.pending) == 0))

  property fairness_grace_period =
    infinitely_often(enabled(start_grace_period)) implies
    infinitely_often(executed(start_grace_period))
}
```

### 15.2 Dependent Types: Safe Array Access

```
module SafeArray {
  type Array<T>(n: Nat) = {
    data: Vec<T>(n)
  }

  fun get<T>(arr: Array<T>(n), i: Nat) -> T
    requires i < n =
    arr.data.data(i)

  fun set<T>(arr: Array<T>(n), i: Nat, val: T) -> Array<T>(n)
    requires i < n =
    mk_Array({data = update(arr.data.data, i, val)}, n)

  // Type checker proves: no out-of-bounds access possible
  theorem bounds_safe:
    forall arr: Array<T>(n), i: Nat:
      i < n implies get(arr, i) is defined
    by auto
}
```

### 15.3 Refinement Types: Division Safety

```
module SafeArithmetic {
  type NonZero = { x: Int | x != 0 }
  type Positive = { x: Int | x > 0 }
  type Even = { n: Nat | n % 2 == 0 }

  fun divide(x: Int, y: NonZero) -> Int =
    x / y.x  // Guaranteed safe

  fun sqrt_approx(x: Positive) -> Positive =
    ... // Implementation

  fun halve(n: Even) -> Nat =
    n / 2  // Guaranteed exact

  // Automatic refinement checking
  proof divide_safe:
    forall x: Int, y: NonZero:
      divide(x, y) is defined
    by smt
}
```

### 15.4 Process Algebra: Two-Phase Commit

```
module TwoPhaseCommit {
  type State = {
    participants: Set<Nat>,
    votes: Map<Nat, Vote>,
    phase: Phase
  } where Phase = Prepare | Commit | Abort

  process Participant(id: Nat, s: State) =
    receive_prepare(s, id) ->
    (vote_yes(s, id) + vote_no(s, id)) ->
    wait_decision(s, id)

  process Coordinator(s: State) =
    broadcast_prepare(s) ->
    collect_votes(s) ->
    (decide_commit(s) + decide_abort(s)) ->
    broadcast_decision(s)

  process System(s: State) =
    Coordinator(s) ||| parallel(p in s.participants) { Participant(p, s) }

  // Temporal properties
  property atomicity =
    always((forall p: committed(p)) or (forall p: aborted(p)))

  property termination =
    eventually(forall p: decided(p))
}
```

### 15.5 Full Example: Seqlock

```
module Seqlock {
  type SeqlockState = {
    seq: Nat,
    data: Int,
    writing: Bool
  } where
    writing implies (seq % 2 == 1)

  // Writer transitions
  transition write_begin(s: SeqlockState) -> SeqlockState
  pre not s.writing
  post s'.seq == s.seq + 1 and
       s'.writing == true and
       s'.data == s.data

  transition write_end(s: SeqlockState, val: Int) -> SeqlockState
  pre s.writing
  post s'.seq == s.seq + 1 and
       s'.writing == false and
       s'.data == val

  // Reader process
  process Reader(s: SeqlockState) : (Int, Bool) =
    let seq0 = s.seq in
    let val = s.data in
    let seq1 = s.seq in
    if seq0 == seq1 and seq0 % 2 == 0
    then (val, true)   // Success
    else Reader(s)     // Retry

  // Invariants
  inv seq_parity(s: SeqlockState) =
    s.writing iff (s.seq % 2 == 1)

  // Temporal properties
  property reader_eventual_success =
    eventually_always(not writing) implies
    eventually(reader_succeeds)

  property writer_progress =
    always(enabled(write_begin) implies eventually(executed(write_end)))
}
```

---

## Appendix A: Complete BNF Grammar

```
program ::= module*

module ::= 'module' ID '{' declaration* '}'

declaration ::=
  | type_def
  | fun_def
  | transition_def
  | process_def
  | invariant
  | property
  | proof
  | export
  | import

type_def ::= 'type' ID type_params? '=' type ('where' expr)?

type ::=
  | base_type
  | ID type_args?
  | '{' record_fields '}'
  | type '|' type                     -- Union
  | '{' ID ':' type '|' expr '}'      -- Refinement
  | '(' ID ':' type ')' '=>' type     -- Dependent function

fun_def ::= 'fun' ID type_params? '(' params ')' '->' type
            ('requires' expr)? ('ensures' expr)? '=' expr

transition_def ::= 'transition' ID '(' params ')' '->' type
                   ('pre' expr)? ('post' expr)? ('preserves' ID (',' ID)*)?

process_def ::= 'process' ID '(' params ')' (':' type)? '=' process_expr

process_expr ::=
  | ID '(' expr_list ')'              -- Process call
  | process_expr '|' process_expr     -- Parallel
  | process_expr '|||' process_expr   -- Interleaving
  | process_expr '->' process_expr    -- Sequence
  | process_expr '+' process_expr     -- Choice
  | process_expr '\' '{' ID_list '}'  -- Restriction

temporal_def ::= 'temporal' ID '(' params ')' '->' type '=' temporal_expr

temporal_expr ::=
  | 'always' '(' expr ')'
  | 'eventually' '(' expr ')'
  | 'next' '(' expr ')'
  | 'until' '(' expr ',' expr ')'
  | ID '(' expr_list ')'

property ::=
  | 'safety' ID '=' temporal_expr
  | 'liveness' ID '=' temporal_expr
  | 'fairness' ID '=' temporal_expr

proof ::= 'proof' ID ':' expr 'by' tactic

tactic ::=
  | 'auto'
  | 'smt'
  | 'model_check' ('depth' '=' NUM)? ('fairness' '=' ID)?
  | 'induction' 'on' ID '{' case* '}'
  | 'unfold' ID
  | tactic ';' tactic

export ::= 'export' ('type' | 'fun' | 'transition' | 'process' | 'inv') ID

import ::= 'import' module_path ('::' '*' | '::' '{' ID_list '}')?
```

---

## Appendix B: Type System Rules

```
Typing judgments: Γ ⊢ e : T

[T-Var]
x : T ∈ Γ
─────────
Γ ⊢ x : T

[T-Abs]
Γ, x : A ⊢ e : B
──────────────────────
Γ ⊢ (λx: A. e) : A → B

[T-App]
Γ ⊢ f : A → B    Γ ⊢ e : A
──────────────────────────────
Γ ⊢ f e : B

[T-Refine]
Γ ⊢ e : T    Γ ⊢ φ[e/x] : Bool
───────────────────────────────
Γ ⊢ e : {x: T | φ}

[T-Dependent]
Γ ⊢ f : (x: A) => B(x)    Γ ⊢ e : A
────────────────────────────────────
Γ ⊢ f e : B(e)

[T-Sub]
Γ ⊢ e : S    S <: T
───────────────────
Γ ⊢ e : T
```

---

## Appendix C: Recommended Reading

1. **CCS**: Milner, R. (1989). *Communication and Concurrency*.
2. **CSP**: Hoare, C. A. R. (1985). *Communicating Sequential Processes*.
3. **Z**: Spivey, J. M. (1992). *The Z Notation: A Reference Manual*.
4. **TLA+**: Lamport, L. (2002). *Specifying Systems*.
5. **LTL**: Pnueli, A. (1977). "The temporal logic of programs".
6. **Lean**: de Moura, L. et al. (2021). *The Lean 4 Theorem Prover*.
7. **Refinement Types**: Rondon, P. et al. (2008). "Liquid Types".
8. **Dependent Types**: Pierce, B. C. (2002). *Types and Programming Languages*.

---

## Appendix D: Tool Support

### VSCode Extension
- Syntax highlighting
- Autocomplete
- Type checking on save
- Inline diagnostics
- Go-to-definition
- Find references

### Model Checker
- BFS/DFS exploration
- LTL/CTL property checking
- Counterexample generation
- DOT graph export
- Fairness constraints

### Proof Assistant
- SMT solver integration (Z3, CVC5)
- Automatic tactic search
- Interactive proof mode
- Proof visualization

---

**End of Specification**
