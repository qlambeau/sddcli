//! Runs the repository's single local quality-gate command.
#![forbid(unsafe_code)]

use std::error::Error;
use std::process::{Command, ExitStatus};

fn main() -> Result<(), Box<dyn Error>> {
    let command = std::env::args().nth(1).ok_or("expected `ci`")?;
    if command != "ci" {
        return Err(format!("unknown xtask command `{command}`").into());
    }

    for gate in gates() {
        run_gate(&gate)?;
    }
    Ok(())
}

struct Gate {
    name: &'static str,
    args: &'static [&'static str],
    rustdoc_warnings: bool,
}

fn gates() -> [Gate; 8] {
    [
        Gate { name: "format", args: &["fmt", "--all", "--", "--check"], rustdoc_warnings: false },
        Gate {
            name: "clippy",
            args: &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
            rustdoc_warnings: false,
        },
        Gate {
            name: "tests",
            args: &["test", "--workspace", "--all-features"],
            rustdoc_warnings: false,
        },
        Gate {
            name: "documentation",
            args: &["doc", "--workspace", "--no-deps"],
            rustdoc_warnings: true,
        },
        Gate { name: "dependency audit", args: &["deny", "check"], rustdoc_warnings: false },
        Gate {
            name: "coverage",
            args: &["llvm-cov", "--workspace", "--fail-under-lines", "85"],
            rustdoc_warnings: false,
        },
        Gate {
            name: "domain coverage",
            args: &["llvm-cov", "--package", "sdd-domain", "--fail-under-lines", "95"],
            rustdoc_warnings: false,
        },
        Gate { name: "release tests", args: &["test", "--release"], rustdoc_warnings: false },
    ]
}

fn run_gate(gate: &Gate) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new("cargo");
    command.args(gate.args);
    if gate.rustdoc_warnings {
        command.env("RUSTDOCFLAGS", "-D warnings");
    }
    let status = command.status()?;
    ensure_success(gate.name, status)
}

fn ensure_success(name: &str, status: ExitStatus) -> Result<(), Box<dyn Error>> {
    if status.success() { Ok(()) } else { Err(format!("{name} gate failed with {status}").into()) }
}
