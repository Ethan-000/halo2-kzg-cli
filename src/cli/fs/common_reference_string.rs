use std::{env, path::PathBuf};

use acvm::{acir::circuit::Circuit, CommonReferenceString};
use halo2_backend::Halo2;
use tokio::runtime::Builder;

use crate::utils::{create_named_dir, write_to_file};

const BACKEND_IDENTIFIER: &str = "halo2-kzg";
const TRANSCRIPT_NAME: &str = "common-reference-string.bin";

fn common_reference_string_location() -> PathBuf {
    let cache_dir = match env::var("NARGO_BACKEND_CACHE_DIR") {
        Ok(cache_dir) => PathBuf::from(cache_dir),
        Err(_) => dirs::home_dir().unwrap().join(".nargo").join("backends"),
    };
    cache_dir.join(BACKEND_IDENTIFIER).join(TRANSCRIPT_NAME)
}

pub(crate) fn read_cached_common_reference_string() -> Vec<u8> {
    let crs_path = common_reference_string_location();

    match std::fs::read(crs_path) {
        Ok(common_reference_string) => common_reference_string,
        Err(_) => vec![],
    }
}

pub(crate) fn update_common_reference_string(
    common_reference_string: &[u8],
    circuit: &Circuit,
) -> Result<Vec<u8>, <Halo2 as CommonReferenceString>::Error> {
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    let fut = if common_reference_string.is_empty() {
        Halo2.generate_common_reference_string(circuit)
    } else {
        Halo2.update_common_reference_string(common_reference_string.to_vec(), circuit)
    };

    runtime.block_on(fut)
}

pub(crate) fn write_cached_common_reference_string(common_reference_string: &[u8]) {
    let crs_path = common_reference_string_location();

    create_named_dir(crs_path.parent().unwrap(), "crs");

    write_to_file(common_reference_string, &crs_path);
}
