use std::path::{Path, PathBuf};

use acvm::ProofSystemCompiler;
use clap::Args;
use halo2_backend::Halo2;
use nargo::artifacts::program::PreprocessedProgram;
use noirc_abi::input_parser::Format;

use crate::{
    constants::{PROOFS_DIR, PROVER_INPUT_FILE, TARGET_DIR, VERIFIER_INPUT_FILE},
    errors::CliError,
    utils::{
        read_inputs_from_file, read_program_from_file, save_proof_to_dir, write_inputs_to_file,
    },
    Halo2Config,
};

use super::{
    execute::execute_program,
    fs::common_reference_string::{
        read_cached_common_reference_string, update_common_reference_string,
        write_cached_common_reference_string,
    },
};

/// Create proof for this program. The proof is returned as a hex encoded string.
#[derive(Debug, Clone, Args)]
pub(crate) struct ProveCommand {
    /// The name of the proof
    proof_name: Option<String>,

    /// The name of the circuit build files (ACIR, proving and verification keys)
    circuit_name: String,

    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, short, default_value = PROVER_INPUT_FILE)]
    prover_name: String,

    /// The name of the toml file which contains the inputs for the verifier
    #[clap(long, short, default_value = VERIFIER_INPUT_FILE)]
    verifier_name: String,
}

pub(crate) fn run(args: ProveCommand, config: Halo2Config) -> Result<(), CliError<Halo2>> {
    let proof_dir = config.program_dir.join(PROOFS_DIR);

    let circuit_build_path = config.program_dir.join(TARGET_DIR).join(args.circuit_name);

    prove_with_path(
        args.proof_name,
        args.prover_name,
        args.verifier_name,
        config.program_dir,
        proof_dir,
        circuit_build_path,
    )?;

    Ok(())
}

pub(crate) fn prove_with_path<P: AsRef<Path>>(
    proof_name: Option<String>,
    prover_name: String,
    verifier_name: String,
    program_dir: P,
    proof_dir: P,
    circuit_build_path: PathBuf,
) -> Result<Option<PathBuf>, CliError<Halo2>> {
    let common_reference_string = read_cached_common_reference_string();

    let preprocessed_program = read_program_from_file(circuit_build_path)?;

    let common_reference_string =
        update_common_reference_string(&common_reference_string, &preprocessed_program.bytecode)
            .map_err(CliError::CommonReferenceStringError)?;

    write_cached_common_reference_string(&common_reference_string);

    let PreprocessedProgram {
        abi,
        bytecode,
        proving_key,
        ..
    } = preprocessed_program;

    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(&program_dir, prover_name.as_str(), Format::Toml, &abi)?;

    let solved_witness = execute_program(bytecode.clone(), &abi, &inputs_map)?;

    // Write public inputs into Verifier.toml
    let public_abi = abi.public_abi();
    let (public_inputs, return_value) = public_abi.decode(&solved_witness)?;

    write_inputs_to_file(
        &public_inputs,
        &return_value,
        &program_dir,
        verifier_name.as_str(),
        Format::Toml,
    )?;

    let proof = Halo2
        .prove_with_pk(
            &common_reference_string,
            &bytecode,
            solved_witness,
            &proving_key,
            false,
        )
        .map_err(CliError::ProofSystemCompilerError)?;

    let proof_path = if let Some(proof_name) = proof_name {
        Some(save_proof_to_dir(&proof, &proof_name, proof_dir)?)
    } else {
        println!("{}", hex::encode(&proof));
        None
    };

    Ok(proof_path)
}
