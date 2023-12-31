use acvm::{
    acir::native_types::WitnessMapError, Backend, CommonReferenceString, ProofSystemCompiler,
    SmartContract,
};
use hex::FromHexError;
use nargo::NargoError;
use noirc_abi::errors::{AbiError, InputParserError};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum FilesystemError {
    #[error("Error: {} is not a valid path\nRun either `nargo compile` to generate missing build artifacts or `nargo prove` to construct a proof", .0.display())]
    PathNotValid(PathBuf),
    #[error("Error: could not parse hex build artifact (proof, proving and/or verification keys, ACIR checksum) ({0})")]
    HexArtifactNotValid(FromHexError),
    #[error(
        " Error: cannot find {0}.toml file.\n Expected location: {1:?} \n Please generate this file at the expected location."
    )]
    MissingTomlFile(String, PathBuf),

    /// Input parsing error
    #[error(transparent)]
    InputParserError(#[from] InputParserError),

    /// WitnessMap serialization error
    #[error(transparent)]
    WitnessMapSerialization(#[from] WitnessMapError),
}

#[derive(Debug, Error)]
pub(crate) enum CliError<B: Backend> {
    #[error("Failed to verify proof {}", .0.display())]
    InvalidProof(PathBuf),

    /// ABI encoding/decoding error
    #[error(transparent)]
    AbiError(#[from] AbiError),

    /// Filesystem errors
    #[error(transparent)]
    FilesystemError(#[from] FilesystemError),

    /// Error from Nargo
    #[error(transparent)]
    NargoError(#[from] NargoError),

    /// Backend error caused by a function on the SmartContract trait
    #[error(transparent)]
    SmartContractError(<B as SmartContract>::Error), // Unfortunately, Rust won't let us `impl From` over an Associated Type on a generic

    /// Backend error caused by a function on the ProofSystemCompiler trait
    #[error(transparent)]
    ProofSystemCompilerError(<B as ProofSystemCompiler>::Error), // Unfortunately, Rust won't let us `impl From` over an Associated Type on a generic

    /// Backend error caused by a function on the CommonReferenceString trait
    #[error(transparent)]
    CommonReferenceStringError(<B as CommonReferenceString>::Error), // Unfortunately, Rust won't let us `impl From` over an Associated Type on a generic
}
