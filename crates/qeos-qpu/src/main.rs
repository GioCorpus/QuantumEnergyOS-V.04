//! qeos-qpu: simulator-backed QPU CLI (Phase 4.3).
//!
//! CLASSIFICATION: SIMULATION. Every command runs the CPU reference or the
//! Majorana model; no hardware is contacted.

use quantum_runtime::{
    execute_on_backend, run_majorana_experiment, BackendCapabilities, ComputeBackendKind,
    ExperimentLimits, MajoranaSimConfig, QuantumCircuit, QuantumExperiment, QuantumGate,
    RuntimeNoiseModel,
};

fn usage() -> String {
    [
        "qeos-qpu (QEOS Phase 4.3 simulator CLI)",
        "Usage: qeos-qpu <command> [options]",
        "  backends                          list simulator backends + capabilities",
        "  capabilities                      show capability flags",
        "  simulate --shots N --seed S       run Bell circuit on cpu-simulator",
        "  majorana --shots N --seed S       run tetron parity experiment",
        "  benchmark                         time 2/4/8-qubit reference runs",
        "  experiment --shots N --seed S     full ExperimentResult JSON",
    ]
    .join("\n")
}

fn arg_value(args: &[String], flag: &str, default: &str) -> String {
    let mut iter = args.iter().peekable();
    while let Some(a) = iter.next() {
        if a == flag {
            return iter.next().cloned().unwrap_or_else(|| default.to_string());
        }
        if let Some(v) = a.strip_prefix(&format!("{flag}=")) {
            return v.to_string();
        }
    }
    default.to_string()
}

fn bell_circuit() -> QuantumCircuit {
    let mut c = QuantumCircuit::new("bell", 2).expect("bell circuit");
    c.add_gate(QuantumGate::Hadamard).expect("h");
    c.add_gate(QuantumGate::CNOT {
        control: 0,
        target: 1,
    })
    .expect("cx");
    c.add_gate(QuantumGate::Measurement { qubit: 0 })
        .expect("m0");
    c.add_gate(QuantumGate::Measurement { qubit: 1 })
        .expect("m1");
    c
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("help");
    match cmd {
        "backends" => {
            let cpu = BackendCapabilities::cpu_simulator(20);
            let maj = BackendCapabilities::majorana_simulator(4);
            println!(
                "{}",
                serde_json::to_string_pretty(&vec![cpu, maj]).expect("json")
            );
        }
        "capabilities" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&BackendCapabilities::majorana_simulator(4))
                    .expect("json")
            );
        }
        "simulate" | "experiment" => {
            let shots: u32 = arg_value(&args, "--shots", "256").parse().unwrap_or(256);
            let seed: u64 = arg_value(&args, "--seed", "42").parse().unwrap_or(42);
            let exp = QuantumExperiment::new(
                format!("qeos-qpu-{cmd}-{seed}"),
                bell_circuit(),
                "cpu-simulator",
                shots,
                RuntimeNoiseModel::ideal(),
                seed,
            );
            match execute_on_backend(&exp, &ExperimentLimits::default(), ComputeBackendKind::Cpu) {
                Ok((result, report)) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "result": result,
                            "compute": report,
                        }))
                        .expect("json")
                    );
                }
                Err(e) => {
                    eprintln!("[ERROR] experiment failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        "majorana" => {
            let shots: u32 = arg_value(&args, "--shots", "1000").parse().unwrap_or(1000);
            let seed: u64 = arg_value(&args, "--seed", "42").parse().unwrap_or(42);
            let res = run_majorana_experiment(
                shots,
                seed,
                &MajoranaSimConfig::default(),
                &RuntimeNoiseModel::ideal(),
            );
            match res {
                Ok(r) => println!("{}", serde_json::to_string_pretty(&r).expect("json")),
                Err(e) => {
                    eprintln!("[ERROR] majorana experiment failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        "benchmark" => {
            for n in [2usize, 4, 8] {
                let mut c = QuantumCircuit::new(format!("bench-{n}"), n).expect("bench");
                for _ in 0..n {
                    c.add_gate(QuantumGate::Hadamard).expect("h");
                }
                let exp = QuantumExperiment::new(
                    format!("bench-{n}"),
                    c,
                    "cpu-simulator",
                    64,
                    RuntimeNoiseModel::ideal(),
                    1,
                );
                let start = std::time::Instant::now();
                let res =
                    execute_on_backend(&exp, &ExperimentLimits::default(), ComputeBackendKind::Cpu);
                match res {
                    Ok((r, _)) => println!(
                        "{{\"qubits\":{n},\"elapsed_ms\":{},\"counts\":{}}}",
                        start.elapsed().as_millis(),
                        r.counts.len()
                    ),
                    Err(e) => eprintln!("bench {n} failed: {e}"),
                }
            }
        }
        _ => println!("{}", usage()),
    }
}
