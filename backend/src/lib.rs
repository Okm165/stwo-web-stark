mod utils;

use cairo_air::{air::CairoProof, verifier::verify_cairo, PreProcessedTraceVariant};
use cairo_vm::{
    cairo_run::{self},
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor,
    types::layout_name::LayoutName,
    vm::{
        errors::cairo_run_errors::CairoRunError,
        runners::{
            cairo_pie::{
                CairoPie, CairoPieAdditionalData, CairoPieMemory, CairoPieMetadata, CairoPieVersion,
            },
            cairo_runner::{ExecutionResources, RunResources},
        },
    },
};
use serde::{Deserialize, Serialize};
use stwo_cairo_adapter::{vm_import::VmImportError, ProverInput};
use stwo_cairo_prover::prover::prove_cairo;
use stwo_prover::core::{
    pcs::PcsConfig,
    prover::ProvingError,
    vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher},
};
use thiserror_no_std::Error;
use utils::{adapt_finished_runner, set_panic_hook};
use wasm_bindgen::prelude::*;

extern crate alloc;

#[cfg(target_arch = "wasm32")]
use lol_alloc::{FreeListAllocator, LockedAllocator};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: LockedAllocator<FreeListAllocator> = LockedAllocator::new(FreeListAllocator::new());

pub struct TraceGenOutput {
    pub execution_resources: ExecutionResources,
    pub prover_input: ProverInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceGenOutputJS {
    execution_resources: String,
    prover_input: String,
}

pub fn from_zip_archive<R: std::io::Read + std::io::Seek>(
    mut zip_reader: zip::ZipArchive<R>,
) -> Result<CairoPie, std::io::Error> {
    use std::io::Read;

    let version = match zip_reader.by_name("version.json") {
        Ok(version_buffer) => {
            let reader = std::io::BufReader::new(version_buffer);
            serde_json::from_reader(reader)?
        }
        Err(_) => CairoPieVersion { cairo_pie: () },
    };

    let reader = std::io::BufReader::new(zip_reader.by_name("metadata.json")?);
    let metadata: CairoPieMetadata = serde_json::from_reader(reader)?;

    let mut memory = vec![];
    zip_reader.by_name("memory.bin")?.read_to_end(&mut memory)?;
    let memory = CairoPieMemory::from_bytes(&memory)
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;

    let reader = std::io::BufReader::new(zip_reader.by_name("execution_resources.json")?);
    let execution_resources: ExecutionResources = serde_json::from_reader(reader)?;

    let reader = std::io::BufReader::new(zip_reader.by_name("additional_data.json")?);
    let additional_data: CairoPieAdditionalData = serde_json::from_reader(reader)?;

    Ok(CairoPie {
        metadata,
        memory,
        execution_resources,
        additional_data,
        version,
    })
}

#[wasm_bindgen]
pub fn run_trace_gen(pie_zip_js: JsValue) -> Result<JsValue, JsValue> {
    set_panic_hook();

    let input: Vec<u8> = serde_wasm_bindgen::from_value(pie_zip_js)?;
    let reader = std::io::Cursor::new(input);
    let zip_archive = zip::ZipArchive::new(reader).unwrap();

    let pie = from_zip_archive(zip_archive)
        .map_err(|e| JsValue::from(format!("Failed to deserialize pie: {e}")))?;
    let trace_gen_output =
        trace_gen(pie).map_err(|e| JsValue::from(format!("Failed to generate trace: {e}")))?;
    Ok(serde_wasm_bindgen::to_value(&TraceGenOutputJS {
        prover_input: serde_json::to_string(&trace_gen_output.prover_input)
            .map_err(|e| JsValue::from(format!("Failed to serialize prover input: {e}")))?,
        execution_resources: serde_json::to_string(&trace_gen_output.execution_resources)
            .map_err(|e| JsValue::from(format!("Failed to serialize execution resources: {e}")))?,
    })?)
}

#[wasm_bindgen]
pub fn run_prove(prover_input_js: JsValue) -> Result<JsValue, JsValue> {
    set_panic_hook();

    let prover_input: ProverInput =
        serde_json::from_str(&serde_wasm_bindgen::from_value::<String>(prover_input_js)?)
            .map_err(|e| JsValue::from(format!("Failed to deserialize prover input: {e}")))?;
    let proof =
        prove(prover_input).map_err(|e| JsValue::from(format!("Failed to generate proof: {e}")))?;
    Ok(serde_wasm_bindgen::to_value(
        &serde_json::to_string(&proof)
            .map_err(|e| JsValue::from(format!("Failed to serialize proof: {e}")))?,
    )?)
}

#[wasm_bindgen]
pub fn run_verify(proof_js: JsValue) -> Result<JsValue, JsValue> {
    set_panic_hook();

    let proof: CairoProof<Blake2sMerkleHasher> =
        serde_json::from_str(&serde_wasm_bindgen::from_value::<String>(proof_js)?)
            .map_err(|e| JsValue::from(format!("Failed to deserialize proof: {e}")))?;
    let verdict = verify(proof);
    Ok(serde_wasm_bindgen::to_value(&verdict)?)
}

pub fn trace_gen(pie: CairoPie) -> Result<TraceGenOutput, VmError> {
    let cairo_run_config = cairo_run::CairoRunConfig {
        trace_enabled: true,
        relocate_mem: true,
        layout: LayoutName::all_cairo_stwo,
        ..Default::default()
    };

    let mut hint_processor = BuiltinHintProcessor::new(
        Default::default(),
        RunResources::new(pie.execution_resources.n_steps),
    );
    let cairo_runner_result =
        cairo_run::cairo_run_pie(&pie, &cairo_run_config, &mut hint_processor);

    let cairo_runner = match cairo_runner_result {
        Ok(runner) => runner,
        Err(error) => {
            return Err(VmError::Runner(error));
        }
    };

    Ok(TraceGenOutput {
        execution_resources: cairo_runner
            .get_execution_resources()
            .map_err(|e| VmError::Runner(CairoRunError::Runner(e)))?,
        prover_input: adapt_finished_runner(cairo_runner)?,
    })
}

pub fn prove(prover_input: ProverInput) -> Result<CairoProof<Blake2sMerkleHasher>, ProvingError> {
    prove_cairo::<Blake2sMerkleChannel>(
        prover_input,
        PcsConfig::default(),
        PreProcessedTraceVariant::CanonicalWithoutPedersen,
    )
}

pub fn verify(cairo_proof: CairoProof<Blake2sMerkleHasher>) -> bool {
    verify_cairo::<Blake2sMerkleChannel>(
        cairo_proof,
        PcsConfig::default(),
        PreProcessedTraceVariant::CanonicalWithoutPedersen,
    )
    .is_ok()
}

#[derive(Debug, Error)]
pub enum VmError {
    #[error("Failed to interact with the file system")]
    IO(#[from] std::io::Error),
    #[error("The cairo program execution failed")]
    Runner(#[from] CairoRunError),
    #[error("The adapter execution failed")]
    Adapter(#[from] VmImportError),
}
