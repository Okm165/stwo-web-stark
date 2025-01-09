mod utils;

use cairo_vm::{
    cairo_run,
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor,
    types::{layout_name::LayoutName, program::Program},
    vm::{errors::cairo_run_errors::CairoRunError, runners::cairo_runner::ExecutionResources},
};
use serde::{Deserialize, Serialize};
use stwo_cairo_prover::{
    cairo_air::{air::CairoProof, prove_cairo, verify_cairo, ProverConfig},
    input::{plain::adapt_finished_runner, ProverInput},
};
use stwo_cairo_utils::vm_utils::VmError;
use stwo_prover::core::{
    prover::ProvingError,
    vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher},
};
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

pub struct TraceGenOutput {
    pub execution_resources: ExecutionResources,
    pub prover_input: ProverInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceGenOutputJS {
    execution_resources: String,
    prover_input: String,
}

#[wasm_bindgen]
pub fn run_trace_gen(program_content_js: JsValue) -> Result<JsValue, JsValue> {
    set_panic_hook();

    let program = Program::from_bytes(
        serde_wasm_bindgen::from_value::<String>(program_content_js)?.as_bytes(),
        None,
    )
    .map_err(|e| JsValue::from(format!("Failed to deserialize program: {e}")))?;
    let trace_gen_output =
        trace_gen(program).map_err(|e| JsValue::from(format!("Failed to generate trace: {e}")))?;
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

pub fn trace_gen(program: Program) -> Result<TraceGenOutput, VmError> {
    let cairo_run_config = cairo_run::CairoRunConfig {
        trace_enabled: true,
        relocate_mem: true,
        layout: LayoutName::all_cairo,
        proof_mode: true,
        ..Default::default()
    };

    let mut hint_processor = BuiltinHintProcessor::new_empty();
    let cairo_runner_result =
        cairo_run::cairo_run_program(&program, &cairo_run_config, &mut hint_processor);

    let cairo_runner = match cairo_runner_result {
        Ok(runner) => runner,
        Err(error) => {
            return Err(VmError::Runner(error.to_string()));
        }
    };

    Ok(TraceGenOutput {
        execution_resources: cairo_runner
            .get_execution_resources()
            .map_err(|e| VmError::Runner(CairoRunError::Runner(e).to_string()))?,
        prover_input: adapt_finished_runner(cairo_runner, false),
    })
}

pub fn prove(prover_input: ProverInput) -> Result<CairoProof<Blake2sMerkleHasher>, ProvingError> {
    prove_cairo::<Blake2sMerkleChannel>(prover_input, ProverConfig::default())
}

pub fn verify(cairo_proof: CairoProof<Blake2sMerkleHasher>) -> bool {
    verify_cairo::<Blake2sMerkleChannel>(cairo_proof).is_ok()
}
