//! A solver for the Travelling Salesman Problem.
//!
//! Based on code developed at ETH by Christoph von Praun, Florian
//! Schneider, Nicholas Matsakis, and Thomas Gross.

use docopt::Docopt;
use std::error::Error;
use std::fmt::Write;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::Instant;

#[cfg(test)]
mod bench;
mod graph;
mod parser;
mod solver;
mod step;
mod tour;
mod weight;

use self::graph::{Graph, Node};
use self::solver::SolverCx;

const USAGE: &str = "
Usage: tsp bench [--seq-threshold N] [--from N] <datafile>

Parallel traveling salesman problem solver. Data input is expected to
be in TSPLIB format.

Suggested command:
    cargo run --release -- tsp bench data/tsp/dj15.tsp --seq-threshold 8

Commands:
    bench              Run the benchmark and print the timings.

Options:
    -h, --help         Show this message.
    --seq-threshold N  Adjust sequential fallback threshold [default: 10].
                       Fall back to seq search when there are N or fewer nodes remaining.
                       Lower values of N mean more parallelism.
    --from N           Node index from which to start the search [default: 0].
";

#[derive(serde::Deserialize)]
pub struct Args {
    cmd_bench: bool,
    arg_datafile: String,
    flag_seq_threshold: usize,
    flag_from: usize,
}

pub fn main(args: &[String]) {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(args).deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.cmd_bench {
        let _ = run_solver(
            Path::new(&args.arg_datafile),
            args.flag_seq_threshold,
            args.flag_from,
        );
    }
}

fn run_solver(datafile: &Path, seq_threshold: usize, from: usize) -> Result<(), ()> {
    let graph = match parse_solver(datafile) {
        Ok(g) => g,
        Err(e) => {
            println!("failed to parse `{}`: {}", datafile.display(), e);
            return Err(());
        }
    };

    println!("Graph size   : {} nodes.", graph.num_nodes());
    println!("Seq threshold: {seq_threshold} nodes.");

    if from >= graph.num_nodes() {
        println!("Invalid node index given for `--from`: {from}");
        return Err(());
    }

    let mut solver = SolverCx::new(&graph, seq_threshold);
    let par_start = Instant::now();
    solver.search_from(Node::new(from));
    let par_time = par_start.elapsed();

    let (path, weight) = solver.into_result();

    println!("Total search time: {par_time:?}");
    if let Some(path) = path {
        println!("Cheapest path cost: {}", weight.to_usize());
        let mut output = "Cheapest path:".to_string();
        for node in path {
            let _ = write!(output, " {}", node.index());
        }
        println!("{output}");
    } else {
        println!("No path found.");
    }

    Ok(())
}

fn parse_solver(datafile: &Path) -> Result<Graph, Box<dyn Error>> {
    let mut file = File::open(datafile)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    let graph = parser::parse_tsp_data(&text)?;
    Ok(graph)
}
