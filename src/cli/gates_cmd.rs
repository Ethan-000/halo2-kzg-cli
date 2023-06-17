use acvm::{ProofSystemCompiler};
use clap::Args;
use halo2_backend::Halo2;
use nargo::artifacts::program::PreprocessedProgram;


use crate::constants::TARGET_DIR;
use crate::errors::CliError;

use crate::utils::read_program_from_file;
use crate::Halo2Config;

/// Counts the occurrences of different gates in circuit
#[derive(Debug, Clone, Args)]
pub(crate) struct GatesCommand {
    /// The name of the circuit build files (ACIR, proving and verification keys)
    circuit_name: String,
}

pub(crate) fn run(args: GatesCommand, config: Halo2Config) -> Result<(), CliError<Halo2>> {
    let circuit_build_path = config.program_dir.join(TARGET_DIR).join(args.circuit_name);
    let preprocessed_program = read_program_from_file(circuit_build_path)?;

    let PreprocessedProgram { bytecode, .. } = preprocessed_program;

    let exact_circuit_size = Halo2
        .get_exact_circuit_size(&bytecode)
        .map_err(CliError::ProofSystemCompilerError)?;

    println!("Backend circuit size: {exact_circuit_size}");

    Ok(())
}
