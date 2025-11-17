# VDL_2026 for VDL/meta-IV Users

**From Compiler Semantics to Concurrent Systems**

---

## Introduction

This document explains VDL_2026 from the perspective of someone familiar with the original Vienna Definition Language (VDL) and its successor meta-IV. VDL_2026 is not a faithful recreation of these languages, but a modern descendant that adapts Vienna principles to a different domain: concurrent system specification with automated state exploration.

---

## Historical Context

### VDL (1960s-1970s)

Created at IBM Vienna Laboratory by Peter Lucas, Kurt Walk, and colleagues for PL/I compiler formal semantics.

```
-- VDL: Tree-based abstract syntax
s-tree(operator, left-subtree, right-subtree)
s-left(tree)
s-right(tree)
is-tree(x)
is-atom(x)

-- Operational semantics via tree rewriting
eval-expr(e, env) =
  if is-atom(e) then lookup(e, env)
  else if s-tag(e) = 'add' then
    eval-expr(s-left(e), env) + eval-expr(s-right(e), env)
  else ...
```

**Key characteristics:**
- Lisp-like tree selectors (s-left, s-right, s-tag)
- Operational style: explicit state transformation
- Focus: compiler/interpreter semantics
- Predicate functions: is-tree, is-atom

### meta-IV (1970s-1980s)

Dines Bjørner and colleagues refined VDL into a typed metalanguage.

```
-- meta-IV: Mathematical domains
Env = Id →m Value
Store = Loc →m Value
State :: store: Store, env: Env

-- Denotational semantics
eval: Expr → Env → Value
eval⟦x⟧ρ = ρ(x)
eval⟦e₁ + e₂⟧ρ = eval⟦e₁⟧ρ + eval⟦e₂⟧ρ

-- Domain constructors
mk-State(store, env)
is-State(x)
inv-State(s) △ dom s.store ∩ dom s.env = {}
```

**Key characteristics:**
- Typed mathematical domains (maps, sets, sequences)
- Denotational semantics: meaning functions
- Domain notation: →m (partial map), →t (total map)
- Implicit state operations: † (override), ‖ (merge)
- Type invariants with inv-

---

## VDL_2026: The Adaptation

VDL_2026 takes Vienna principles into a new domain: concurrent systems with automated state-space exploration.

### Domain Shift

| Aspect | VDL/meta-IV | VDL_2026 |
|--------|-------------|----------|
| **Primary domain** | Programming language semantics | Concurrent system behavior |
| **State** | Abstract syntax trees, environments | System configurations |
| **Transitions** | Semantic equations | Named state transformations |
| **Verification** | Manual proof | Automated exploration |
| **Output** | Meaning (value) | Invariant violations + traces |

### What VDL_2026 Preserves

**1. Vienna Prefix Conventions**

```
-- meta-IV
mk-State(store, env)          -- constructor
is-State(x)                   -- type test
inv-State(s)                  -- invariant

-- VDL_2026
mk_rcu_state(readers, epoch)  -- constructor
is_nat(x)                     -- type test
inv_callback_safety(s)        -- invariant
```

**2. Record/Composite Types**

```
-- meta-IV
State :: store: Store
         env: Env

s.store                       -- field access

-- VDL_2026
type RcuState = {
  readers: Set<Nat>,
  epoch: Nat
}

s.readers                     -- field access
```

**3. Mathematical Collections**

```
-- meta-IV
dom m                         -- domain of map
rng m                         -- range of map
m † [x ↦ v]                   -- map override
s₁ ∪ s₂                       -- set union
⟨a, b, c⟩                     -- sequence

-- VDL_2026
dom(m)                        -- domain of map
rng(m)                        -- range of map
m with [x |-> v]              -- map override
s1 union s2                   -- set union
[a, b, c]                     -- list
```

**4. Quantified Expressions**

```
-- meta-IV
∀x ∈ S · P(x)
∃x ∈ S · P(x)

-- VDL_2026
forall x in S: P(x)
exists x in S: P(x)
```

**5. Let Bindings**

```
-- meta-IV
let v = e₁ in e₂

-- VDL_2026
let v = e1 in e2
```

### What VDL_2026 Changes

**1. Explicit Named Transitions**

meta-IV defines meaning through semantic equations:

```
-- meta-IV: Denotational style
eval⟦while b do s⟧σ =
  if eval⟦b⟧σ then eval⟦while b do s⟧(eval⟦s⟧σ)
  else σ
```

VDL_2026 uses named operational transitions:

```
-- VDL_2026: Named transitions
transition rcu_read_lock(s: RcuState, r: Nat) -> RcuState
pre r not_in s.readers
post s'.readers = s.readers union {r}

transition rcu_read_unlock(s: RcuState, r: Nat) -> RcuState
pre r in s.readers
post s'.readers = s.readers \ {r}
```

This shift supports automated exploration: each transition is a discrete, named step in the state graph.

**2. Pre/Post Style Instead of Equations**

meta-IV uses functional equations:

```
-- meta-IV
assign⟦x := e⟧σ = σ † [σ.env(x) ↦ eval⟦e⟧σ]
```

VDL_2026 uses pre/post specifications:

```
-- VDL_2026
transition assign(s: State, x: Id, v: Value) -> State
pre x in dom(s.env)
post s'.store = s.store with [s.env(x) |-> v]
```

**3. Invariants as Runtime Checks**

meta-IV invariants are type constraints:

```
-- meta-IV: Type invariant
inv-State(mk-State(store, env)) △
  dom store ∩ dom env = {}
```

VDL_2026 invariants are runtime-checkable properties:

```
-- VDL_2026: Checked during exploration
inv callback_safety(s: RcuState) =
  forall cb in s.pending:
    cb.epoch <= s.epoch
```

The explorer evaluates invariants at each state, producing violation traces.

**4. Automated State-Space Exploration**

meta-IV semantics are for human proof and understanding:

```
-- meta-IV: You prove properties manually
-- Show: ∀s · inv-State(s) ⇒ inv-State(step(s))
```

VDL_2026 explores automatically:

```bash
$ vdl_2026 rcu --depth 5 --readers 2
States explored: 190
Unique states: 45
Invariant violations: 98
```

---

## Side-by-Side: Defining a Simple Language

### meta-IV Approach

```
-- Abstract syntax
Prog = Stmt*
Stmt = Assign | If | While
Assign :: var: Id, expr: Expr
If :: test: Expr, then: Stmt, else: Stmt
While :: test: Expr, body: Stmt

-- Semantic domains
Env = Id →m Loc
Store = Loc →m Value
State :: env: Env, store: Store

-- Meaning functions
eval-prog: Prog → State → State
eval-prog⟦⟨⟩⟧σ = σ
eval-prog⟦⟨s⟩ ^ ss⟧σ = eval-prog⟦ss⟧(eval-stmt⟦s⟧σ)

eval-stmt: Stmt → State → State
eval-stmt⟦mk-Assign(x, e)⟧σ =
  σ † [.store ↦ σ.store † [σ.env(x) ↦ eval-expr⟦e⟧σ]]
```

### VDL_2026 Approach

```
-- Type definitions
type State = {
  vars: Map<Id, Value>,
  pc: Nat
}

-- Individual transitions
transition assign(s: State, x: Id, v: Value) -> State
pre x in dom(s.vars)
post s'.vars = s.vars with [x |-> v] and
     s'.pc = s.pc + 1

transition branch(s: State, cond: Bool, target: Nat) -> State
pre true
post s'.pc = if cond then target else s.pc + 1 and
     s'.vars = s.vars

-- Invariants
inv vars_bounded(s: State) =
  forall x in dom(s.vars):
    s.vars(x) < 1000

inv pc_valid(s: State) =
  s.pc < program_length
```

---

## Mathematical Notation Translation

### Domain Expressions

| meta-IV | VDL_2026 | Meaning |
|---------|----------|---------|
| `S →m T` | `Map<S, T>` | Partial map |
| `S-set` | `Set<S>` | Finite set |
| `S*` | `List<S>` | Sequence |
| `S × T` | `{a: S, b: T}` | Product (as record) |
| `S \| T` | Not supported | Union type |

### Set Operations

| meta-IV | VDL_2026 | Meaning |
|---------|----------|---------|
| `s₁ ∪ s₂` | `s1 union s2` | Union |
| `s₁ ∩ s₂` | `s1 intersect s2` | Intersection |
| `s₁ \ s₂` | `s1 \ s2` | Difference |
| `x ∈ s` | `x in s` | Membership |
| `x ∉ s` | `x not_in s` | Non-membership |
| `card s` | `card(s)` | Cardinality |
| `{x ∈ S \| P(x)}` | `{x in S \| P(x)}` | Comprehension |

### Map Operations

| meta-IV | VDL_2026 | Meaning |
|---------|----------|---------|
| `dom m` | `dom(m)` | Domain |
| `rng m` | `rng(m)` | Range |
| `m(x)` | `m(x)` | Application |
| `m † [x ↦ v]` | `m with [x \|-> v]` | Override |
| `m₁ ‖ m₂` | `m1 munion m2` | Merge (disjoint) |

### Sequence Operations

| meta-IV | VDL_2026 | Meaning |
|---------|----------|---------|
| `⟨a, b, c⟩` | `[a, b, c]` | Literal |
| `hd s` | `head(s)` | First element |
| `tl s` | `tail(s)` | Rest |
| `s₁ ^ s₂` | `s1 ++ s2` | Concatenation |
| `len s` | `len(s)` | Length |
| `s(i)` | `s(i)` | Index (1-based) |
| `elems s` | `elems(s)` | Set of elements |

### Quantifiers

| meta-IV | VDL_2026 | Meaning |
|---------|----------|---------|
| `∀x ∈ S · P` | `forall x in S: P` | Universal |
| `∃x ∈ S · P` | `exists x in S: P` | Existential |

---

## What VDL_2026 Cannot Express

### 1. Recursive Semantic Equations

meta-IV naturally expresses recursive definitions:

```
-- meta-IV: Natural recursion
eval⟦while b do s⟧σ =
  if eval⟦b⟧σ
  then eval⟦while b do s⟧(eval⟦s⟧σ)
  else σ
```

VDL_2026 has no direct equivalent. Loops must be unrolled into transition sequences.

### 2. Higher-Order Semantics

meta-IV supports meaning functions as first-class:

```
-- meta-IV
Cont = State → Result
eval-expr: Expr → Env → Cont → Result
```

VDL_2026 functions are limited to value manipulation.

### 3. Union/Variant Types

meta-IV has algebraic data types:

```
-- meta-IV
Value = Num | Bool | Loc
```

VDL_2026 uses records only; no native sum types.

### 4. Implicit State Threading

meta-IV can use implicit monadic state:

```
-- meta-IV with implicit state
assign(x, v); eval(e)
-- state flows implicitly
```

VDL_2026 requires explicit state in every transition signature.

---

## Why These Changes?

### 1. Automation Over Elegance

meta-IV optimizes for human understanding of language semantics. VDL_2026 optimizes for machine exploration of system behavior. Named transitions with pre/post conditions are more tractable for BFS exploration than recursive semantic equations.

### 2. Concurrent Systems Focus

VDL/meta-IV defined sequential language constructs. VDL_2026 models concurrent interleavings (RCU readers, grace periods, callbacks). The transition model naturally captures non-deterministic choice.

### 3. Counterexample Generation

When invariants fail, VDL_2026 produces traces:

```
Violation: callback_safety
Trace:
  1. initial_state
  2. rcu_read_lock(reader=1)
  3. call_rcu(callback)
  4. start_grace_period    // Invariant violated here
```

This requires operational, step-by-step semantics rather than denotational meaning.

### 4. IMPLICIT NONE Philosophy

VDL_2026's mission is making hidden assumptions explicit. Concurrent bugs often arise from implicit ordering, implicit visibility, implicit synchronization. Named transitions with explicit pre/post conditions surface these assumptions.

---

## Adapting meta-IV Specifications to VDL_2026

### Example: Simple Variable Store

**meta-IV version:**

```
Store = Id →m Value
State :: store: Store

inv-State(mk-State(s)) △
  ∀x ∈ dom s · is-Num(s(x)) ⇒ s(x) ≥ 0

assign: Id × Value → State → State
assign(x, v)(σ) = mk-State(σ.store † [x ↦ v])

lookup: Id → State → Value
lookup(x)(σ) = σ.store(x)
```

**VDL_2026 adaptation:**

```
type State = {
  store: Map<Id, Value>
}

inv non_negative(s: State) =
  forall x in dom(s.store):
    is_nat(s.store(x)) implies s.store(x) >= 0

transition assign(s: State, x: Id, v: Value) -> State
pre true
post s'.store = s.store with [x |-> v]

fun lookup(s: State, x: Id) -> Value =
  s.store(x)
```

Note: The pure function `lookup` doesn't modify state, while `assign` is a transition.

---

## Conclusion

VDL_2026 is not VDL or meta-IV reborn. It's a new language that:

1. **Borrows** Vienna naming conventions (mk_, is_, inv_)
2. **Adapts** mathematical domain notation (sets, maps, records)
3. **Preserves** the spirit of explicit, rigorous specification
4. **Changes** the domain from language semantics to concurrent systems
5. **Adds** automated state-space exploration with invariant checking

For VDL/meta-IV practitioners: think of VDL_2026 as applying Vienna rigor to a new problem domain, not as extending the original metalanguage. The surface syntax is modernized, the semantics shifted from denotational equations to operational transitions, and the verification moved from manual proof to automated exploration.

The IMPLICIT NONE philosophy—making hidden assumptions explicit—is the thread connecting VDL_2026 back to the Vienna tradition of formal precision.

---

## References

- Lucas, P., Lauer, P., & Stigleitner, H. (1968). *Method and notation for the formal definition of programming languages*. IBM Vienna Laboratory Technical Report TR 25.087.
- Bjørner, D., & Jones, C. B. (Eds.). (1978). *The Vienna Development Method: The Meta-Language*. Lecture Notes in Computer Science, Vol. 61.
- Jones, C. B. (1990). *Systematic Software Development Using VDM*. Prentice Hall.
