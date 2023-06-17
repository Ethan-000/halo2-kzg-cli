use std::{
    collections::BTreeMap,
    fs::{File, ReadDir},
    io::Write,
    path::{Path, PathBuf},
};

use nargo::{artifacts::program::PreprocessedProgram, manifest::InvalidPackageError};
use noirc_abi::{
    input_parser::{Format, InputValue},
    Abi, InputMap, MAIN_RETURN_NAME,
};

use crate::{constants::PROOF_EXT, errors::FilesystemError};

/// Returns the path of the root directory of the package containing `current_path`.
///
/// Returns a `CliError` if no parent directories of `current_path` contain a manifest file.
pub(crate) fn find_package_root(current_path: &Path) -> Result<PathBuf, InvalidPackageError> {
    let manifest_path = find_package_manifest(current_path)?;

    let package_root = manifest_path
        .parent()
        .expect("infallible: manifest file path can't be root directory");

    Ok(package_root.to_path_buf())
}

/// Returns the path of the manifest file (`Nargo.toml`) of the package containing `current_path`.
///
/// Returns a `CliError` if no parent directories of `current_path` contain a manifest file.
fn find_package_manifest(current_path: &Path) -> Result<PathBuf, InvalidPackageError> {
    Ok(current_path
        .ancestors()
        .find_map(|dir| find_file(dir, "Nargo", "toml"))
        .unwrap())
}

// Looks for file named `file_name` in path
fn find_file<P: AsRef<Path>>(path: P, file_name: &str, extension: &str) -> Option<PathBuf> {
    let entries = list_files_and_folders_in(path)?;
    let file_name = format!("{file_name}.{extension}");

    find_artifact(entries, &file_name)
}

// There is no distinction between files and folders
fn find_artifact(entries: ReadDir, artifact_name: &str) -> Option<PathBuf> {
    let entry = entries
        .into_iter()
        .flatten()
        .find(|entry| entry.file_name().to_str() == Some(artifact_name))?;

    Some(entry.path())
}

fn list_files_and_folders_in<P: AsRef<Path>>(path: P) -> Option<ReadDir> {
    std::fs::read_dir(path).ok()
}

pub(super) fn create_named_dir(named_dir: &Path, name: &str) -> PathBuf {
    std::fs::create_dir_all(named_dir)
        .unwrap_or_else(|_| panic!("could not create the `{name}` directory"));

    PathBuf::from(named_dir)
}

pub(super) fn write_to_file(bytes: &[u8], path: &Path) -> String {
    let display = path.display();

    let mut file = match File::create(path) {
        Err(why) => panic!("couldn't create {display}: {why}"),
        Ok(file) => file,
    };

    match file.write_all(bytes) {
        Err(why) => panic!("couldn't write to {display}: {why}"),
        Ok(_) => display.to_string(),
    }
}

pub(crate) fn read_program_from_file<P: AsRef<Path>>(
    circuit_path: P,
) -> Result<PreprocessedProgram, FilesystemError> {
    let file_path = circuit_path.as_ref().with_extension("json");

    let input_string =
        std::fs::read(&file_path).map_err(|_| FilesystemError::PathNotValid(file_path))?;

    let program = serde_json::from_slice(&input_string).expect("could not deserialize program");

    Ok(program)
}

/// Returns the circuit's parameters and its return value, if one exists.
/// # Examples
///
/// ```ignore
/// let (input_map, return_value): (InputMap, Option<InputValue>) =
///   read_inputs_from_file(path, "Verifier", Format::Toml, &abi)?;
/// ```
pub(crate) fn read_inputs_from_file<P: AsRef<Path>>(
    path: P,
    file_name: &str,
    format: Format,
    abi: &Abi,
) -> Result<(InputMap, Option<InputValue>), FilesystemError> {
    if abi.is_empty() {
        return Ok((BTreeMap::new(), None));
    }

    let file_path = path.as_ref().join(file_name).with_extension(format.ext());
    if !file_path.exists() {
        return Err(FilesystemError::MissingTomlFile(
            file_name.to_owned(),
            file_path,
        ));
    }

    let input_string = std::fs::read_to_string(file_path).unwrap();
    let mut input_map = format.parse(&input_string, abi)?;
    let return_value = input_map.remove(MAIN_RETURN_NAME);

    Ok((input_map, return_value))
}

pub(crate) fn write_inputs_to_file<P: AsRef<Path>>(
    input_map: &InputMap,
    return_value: &Option<InputValue>,
    path: P,
    file_name: &str,
    format: Format,
) -> Result<(), FilesystemError> {
    let file_path = path.as_ref().join(file_name).with_extension(format.ext());

    // We must insert the return value into the `InputMap` in order for it to be written to file.
    let serialized_output = match return_value {
        // Parameters and return values are kept separate except for when they're being written to file.
        // As a result, we don't want to modify the original map and must clone it before insertion.
        Some(return_value) => {
            let mut input_map = input_map.clone();
            input_map.insert(MAIN_RETURN_NAME.to_owned(), return_value.clone());
            format.serialize(&input_map)?
        }
        // If no return value exists, then we can serialize the original map directly.
        None => format.serialize(input_map)?,
    };

    write_to_file(serialized_output.as_bytes(), &file_path);

    Ok(())
}

pub(crate) fn save_proof_to_dir<P: AsRef<Path>>(
    proof: &[u8],
    proof_name: &str,
    proof_dir: P,
) -> Result<PathBuf, FilesystemError> {
    create_named_dir(proof_dir.as_ref(), "proof");
    let proof_path = proof_dir
        .as_ref()
        .join(proof_name)
        .with_extension(PROOF_EXT);

    write_to_file(hex::encode(proof).as_bytes(), &proof_path);

    Ok(proof_path)
}

pub(crate) fn load_hex_data<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, FilesystemError> {
    let hex_data: Vec<_> = std::fs::read(&path)
        .map_err(|_| FilesystemError::PathNotValid(path.as_ref().to_path_buf()))?;

    let raw_bytes = hex::decode(hex_data).map_err(FilesystemError::HexArtifactNotValid)?;

    Ok(raw_bytes)
}
