use super::fs::common_reference_string::{
    read_cached_common_reference_string, update_common_reference_string,
    write_cached_common_reference_string,
};
use crate::utils::{load_hex_data, read_inputs_from_file, read_program_from_file};
use crate::Halo2Config;
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, TARGET_DIR, VERIFIER_INPUT_FILE},
    errors::CliError,
};

use acvm::ProofSystemCompiler;
use clap::Args;
use halo2_backend::Halo2;
use nargo::artifacts::program::PreprocessedProgram;

use noirc_abi::input_parser::Format;
use std::path::{Path, PathBuf};

/// Given a proof and a program, verify whether the proof is valid
#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyCommand {
    /// The proof to verify
    proof: String,

    /// The name of the circuit build files (ACIR, proving and verification keys)
    circuit_name: String,

    /// The name of the toml file which contains the inputs for the verifier
    #[clap(long, short, default_value = VERIFIER_INPUT_FILE)]
    verifier_name: String,
}

pub(crate) fn run(args: VerifyCommand, config: Halo2Config) -> Result<(), CliError<Halo2>> {
    let proof_path = config
        .program_dir
        .join(PROOFS_DIR)
        .join(&args.proof)
        .with_extension(PROOF_EXT);

    let circuit_build_path = config.program_dir.join(TARGET_DIR).join(args.circuit_name);

    verify_with_path(
        &config.program_dir,
        proof_path,
        &circuit_build_path,
        args.verifier_name,
    )
}

fn verify_with_path<P: AsRef<Path>>(
    program_dir: P,
    proof_path: PathBuf,
    circuit_build_path: P,
    verifier_name: String,
) -> Result<(), CliError<Halo2>> {
    let common_reference_string = read_cached_common_reference_string();

    let preprocessed_program = read_program_from_file(circuit_build_path)?;

    let common_reference_string =
        update_common_reference_string(&common_reference_string, &preprocessed_program.bytecode)
            .map_err(CliError::CommonReferenceStringError)?;

    write_cached_common_reference_string(&common_reference_string);

    let PreprocessedProgram {
        abi,
        bytecode,
        verification_key,
        ..
    } = preprocessed_program;

    // Load public inputs (if any) from `verifier_name`.
    let public_abi = abi.public_abi();
    let (public_inputs_map, return_value) = read_inputs_from_file(
        program_dir,
        verifier_name.as_str(),
        Format::Toml,
        &public_abi,
    )?;

    let public_inputs = public_abi.encode(&public_inputs_map, return_value)?;
    let proof = load_hex_data(&proof_path)?;

    let valid_proof = Halo2
        .verify_with_vk(
            &common_reference_string,
            &proof,
            public_inputs,
            &bytecode,
            &verification_key,
            false,
        )
        .map_err(CliError::ProofSystemCompilerError)?;

    if valid_proof {
        Ok(())
    } else {
        Err(CliError::InvalidProof(proof_path))
    }
}
