use acvm::{
    acir::{circuit::Circuit, native_types::WitnessMap},
    pwg::{block::Blocks, solve, PartialWitnessGeneratorStatus},
};
use halo2_backend::Halo2;
use nargo::NargoError;
use noirc_abi::{Abi, InputMap};

use crate::errors::CliError;

pub(crate) fn execute_program(
    circuit: Circuit,
    abi: &Abi,
    inputs_map: &InputMap,
) -> Result<WitnessMap, CliError<Halo2>> {
    let initial_witness = abi.encode(inputs_map, None)?;

    let solved_witness = execute_circuit(circuit, initial_witness)?;

    Ok(solved_witness)
}

pub fn execute_circuit(
    circuit: Circuit,
    mut initial_witness: WitnessMap,
) -> Result<WitnessMap, NargoError> {
    let mut blocks = Blocks::default();
    let solver_status = solve(&Halo2, &mut initial_witness, &mut blocks, circuit.opcodes)?;
    if matches!(
        solver_status,
        PartialWitnessGeneratorStatus::RequiresOracleData { .. }
    ) {
        todo!("Add oracle support to nargo execute")
    }

    Ok(initial_witness)
}
