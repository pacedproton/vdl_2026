//! VDL_2026 CLI - Vienna Definition Language 2026
//!
//! IMPLICIT NONE for kernel concurrency.
//! Command-line interface for VDL_2026 state-space exploration and verification.

use clap::{Parser, Subcommand};
use std::collections::BTreeSet;
use std::path::PathBuf;
use vdl_2026::explorer::{generate_dot, ExplorerConfig};
use vdl_2026::models::rcu::{make_rcu_state, RcuExplorer, RCU_VDL_SOURCE};

#[derive(Parser)]
#[command(name = "vdl_2026")]
#[command(about = "VDL_2026 - IMPLICIT NONE for kernel concurrency")]
#[command(long_about = r#"
VDL_2026 - Vienna Definition Language 2026

IMPLICIT NONE for kernel concurrency.

Where implicit assumptions become explicit specifications.
Where hidden state transitions become visible invariants.
Where subtle bugs become counterexample traces.

Heritage: IBM Vienna Laboratory → VDL → meta-IV → VDM-SL → VDL_2026

Flagship: Linux kernel RCU (Read-Copy-Update) formal semantics.
"#)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Explore RCU state space
    Rcu {
        /// Maximum exploration depth
        #[arg(short, long, default_value = "6")]
        depth: usize,

        /// Maximum number of states to explore
        #[arg(short = 's', long, default_value = "1000")]
        max_states: usize,

        /// Number of reader IDs to model
        #[arg(short, long, default_value = "2")]
        readers: usize,

        /// Number of callback IDs to model
        #[arg(short, long, default_value = "1")]
        callbacks: usize,

        /// Output DOT graph to file
        #[arg(short, long)]
        graph: Option<PathBuf>,

        /// Output JSON trace to file
        #[arg(short, long)]
        json: Option<PathBuf>,

        /// Stop on first violation
        #[arg(long)]
        stop_on_violation: bool,
    },

    /// Show the VDL_2026 source for RCU model
    ShowRcu,

    /// Parse a VDL_2026 specification file
    Parse {
        /// Path to VDL_2026 source file
        file: PathBuf,
    },

    /// Display version and heritage information
    Info,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Rcu {
            depth,
            max_states,
            readers,
            callbacks,
            graph,
            json,
            stop_on_violation,
        } => {
            run_rcu_exploration(
                depth,
                max_states,
                readers,
                callbacks,
                graph,
                json,
                stop_on_violation,
            );
        }

        Commands::ShowRcu => {
            println!("{}", RCU_VDL_SOURCE);
        }

        Commands::Parse { file } => {
            parse_file(file);
        }

        Commands::Info => {
            print_info();
        }
    }
}

fn run_rcu_exploration(
    depth: usize,
    max_states: usize,
    num_readers: usize,
    num_callbacks: usize,
    graph_output: Option<PathBuf>,
    json_output: Option<PathBuf>,
    stop_on_violation: bool,
) {
    println!("VDL_2026 RCU State-Space Explorer");
    println!("===================================");
    println!("IMPLICIT NONE: All state transitions explicit");
    println!();
    println!("Configuration:");
    println!("  Max depth: {}", depth);
    println!("  Max states: {}", max_states);
    println!("  Reader IDs: 1..{}", num_readers);
    println!("  Callback IDs: 1..{}", num_callbacks);
    println!();

    let config = ExplorerConfig {
        max_depth: depth,
        max_states,
        stop_on_first_violation: stop_on_violation,
        generate_graph: graph_output.is_some() || json_output.is_some(),
        record_all_traces: false,
    };

    let explorer = RcuExplorer::new(config);

    // Initial RCU state
    let initial_state = make_rcu_state(
        BTreeSet::new(), // No readers initially
        0,               // Epoch 0
        false,           // No grace period active
        vec![],          // No pending callbacks
        vec![],          // No completed callbacks
    );

    let reader_ids: Vec<u64> = (1..=num_readers as u64).collect();
    let callback_ids: Vec<u64> = (1..=num_callbacks as u64).collect();

    println!("Starting exploration...");
    let result = explorer.explore(initial_state, reader_ids, callback_ids);

    println!();
    println!("Exploration Results:");
    println!("====================");
    println!("  Visited states: {}", result.visited_states);
    println!("  Max depth reached: {}", result.max_depth_reached);
    println!("  Invariant violations: {}", result.violations.len());

    if !result.violations.is_empty() {
        println!();
        println!("VIOLATIONS FOUND:");
        for (i, violation) in result.violations.iter().enumerate() {
            println!("  {}. {}", i + 1, violation.message);
            println!("     Invariant: {}", violation.invariant_name);
            println!("     At step: {}", violation.step_index);
        }

        // Show trace to first violation
        if let Some(trace) = result.traces.first() {
            println!();
            println!("Trace to first violation:");
            for (i, step) in trace.steps.iter().enumerate() {
                let args_str = step
                    .args
                    .iter()
                    .map(|v| v.pretty())
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("  {}. {}({})", i + 1, step.transition_name, args_str);
            }
        }
    } else {
        println!();
        println!("No invariant violations found.");
        println!("The RCU model appears to be correct within the explored state space.");
    }

    // Output DOT graph
    if let Some(path) = graph_output {
        let dot = generate_dot(&result.graph);
        match std::fs::write(&path, dot) {
            Ok(_) => println!("\nGraph written to: {}", path.display()),
            Err(e) => eprintln!("Error writing graph: {}", e),
        }
    }

    // Output JSON
    if let Some(path) = json_output {
        match serde_json::to_string_pretty(&result) {
            Ok(json) => match std::fs::write(&path, json) {
                Ok(_) => println!("JSON output written to: {}", path.display()),
                Err(e) => eprintln!("Error writing JSON: {}", e),
            },
            Err(e) => eprintln!("Error serializing JSON: {}", e),
        }
    }

    println!();
    println!("State space graph:");
    println!("  Nodes: {}", result.graph.nodes.len());
    println!("  Edges: {}", result.graph.edges.len());
}

fn parse_file(file: PathBuf) {
    match std::fs::read_to_string(&file) {
        Ok(source) => match vdl_2026::parse(&source) {
            Ok(spec) => {
                println!("Successfully parsed: {}", file.display());
                println!();
                println!("Specification:");
                println!("  Types: {}", spec.type_defs.len());
                for td in &spec.type_defs {
                    println!("    - {}", td.name);
                }
                println!("  Functions: {}", spec.functions.len());
                for f in &spec.functions {
                    println!("    - {}", f.name);
                }
                println!("  Transitions: {}", spec.transitions.len());
                for t in &spec.transitions {
                    println!("    - {}", t.name);
                }
                println!("  Invariants: {}", spec.invariants.len());
                for inv in &spec.invariants {
                    println!("    - {}", inv.name);
                }
            }
            Err(e) => {
                eprintln!("Parse error: {}", e);
            }
        },
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }
}

fn print_info() {
    println!(
        r#"
VDL_2026 - Vienna Definition Language 2026
Version: {}

IMPLICIT NONE for kernel concurrency.

Making the implicit explicit.
Where hidden assumptions become visible specifications.
Where subtle bugs become counterexample traces.

HERITAGE:
=========
VDL_2026 honors the IBM Vienna Laboratory tradition:

  1970s: VDL (Vienna Definition Language)
         - Used for PL/I formal semantics
         - Operational semantic metalanguage
         - mk_, is_, inv_ prefixes

  1980s: meta-IV
         - Cleaned-up typed successor to VDL
         - Denotational semantics core
         - Pure mathematical domains

  1990s: VDM-SL (Vienna Development Method Spec Language)
         - BSI standard formal specification language
         - Built on meta-IV semantics

  2026:  VDL_2026
         - IMPLICIT NONE philosophy
         - Explicit state semantics for concurrent systems
         - Flagship: Linux kernel RCU modeling
         - No hidden transitions, no implicit assumptions

PHILOSOPHY:
===========
Like Fortran's IMPLICIT NONE, VDL_2026 demands:
  - Every state transition is explicit
  - Every invariant is checkable
  - Every assumption is visible
  - Every violation has a trace

FEATURES:
=========
- Denotational semantics for types, expressions, invariants
- Operational semantics for state transitions
- Bounded state-space exploration (BFS/DFS)
- Invariant checking and violation detection
- GraphViz DOT output for visualization
- JSON trace export

USE CASES:
==========
- RCU (Read-Copy-Update) protocol semantics
- Concurrent data structure invariants
- API contract verification
- UAPI/syscall semantics modeling
- Driver/hardware register protocols

COMMANDS:
=========
  vdl_2026 rcu          Explore RCU state space
  vdl_2026 show-rcu     Display RCU VDL_2026 model source
  vdl_2026 parse <file> Parse a VDL_2026 specification
  vdl_2026 info         Show this information

EXAMPLE:
========
  $ vdl_2026 rcu --depth 8 --readers 3 --graph rcu.dot
  $ dot -Tsvg rcu.dot > rcu.svg

For the Vienna Lab alumni: IMPLICIT NONE lives again.
"#,
        vdl_2026::VERSION
    );
}
