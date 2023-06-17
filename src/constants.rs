// Directories
/// The directory for the `nargo contract` command output
pub(crate) const CONTRACT_DIR: &str = "contract";
/// The directory to store serialized circuit proofs.
pub(crate) const PROOFS_DIR: &str = "proofs";
/// The directory to store circuits' serialized ACIR representations.
pub(crate) const TARGET_DIR: &str = "target";

// Files
/// The file from which Nargo pulls prover inputs
pub(crate) const PROVER_INPUT_FILE: &str = "Prover";
/// The file from which Nargo pulls verifier inputs
pub(crate) const VERIFIER_INPUT_FILE: &str = "Verifier";

// Extensions
/// The extension for files containing circuit proofs.
pub(crate) const PROOF_EXT: &str = "proof";
