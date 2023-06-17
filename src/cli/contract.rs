use super::fs::common_reference_string::{
    read_cached_common_reference_string, update_common_reference_string,
    write_cached_common_reference_string,
};
use crate::constants::CONTRACT_DIR;
use crate::utils::{create_named_dir, read_program_from_file, write_to_file};
use crate::Halo2Config;
use crate::{constants::TARGET_DIR, errors::CliError};
use acvm::SmartContract;
use clap::Args;
use halo2_backend::Halo2;

/// Generates a Solidity verifier smart contract for the program
#[derive(Debug, Clone, Args)]
pub(crate) struct ContractCommand {
    /// The name of the circuit build files (ACIR, proving and verification keys)
    circuit_name: String,
}

pub(crate) fn run(args: ContractCommand, config: Halo2Config) -> Result<(), CliError<Halo2>> {
    let circuit_build_path = config.program_dir.join(TARGET_DIR).join(args.circuit_name);

    let common_reference_string = read_cached_common_reference_string();

    let preprocessed_program = read_program_from_file(circuit_build_path)?;

    let common_reference_string =
        update_common_reference_string(&common_reference_string, &preprocessed_program.bytecode)
            .map_err(CliError::CommonReferenceStringError)?;

    let smart_contract_string = Halo2
        .eth_contract_from_vk(
            &common_reference_string,
            &preprocessed_program.verification_key,
        )
        .map_err(CliError::SmartContractError)?;

    write_cached_common_reference_string(&common_reference_string);

    let contract_dir = config.program_dir.join(CONTRACT_DIR);
    create_named_dir(&contract_dir, "contract");
    let contract_path = contract_dir.join("plonk_vk").with_extension("sol");

    let path = write_to_file(smart_contract_string.as_bytes(), &contract_path);
    println!("Contract successfully created and located at {path}");
    Ok(())
}
