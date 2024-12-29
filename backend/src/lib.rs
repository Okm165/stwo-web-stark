mod utils;

use cairo_vm::{
    cairo_run,
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor,
    types::layout_name::LayoutName,
    vm::{errors::cairo_run_errors::CairoRunError, runners::cairo_runner::ExecutionResources},
};
use serde::{Deserialize, Serialize};
use stwo_cairo_prover::{
    cairo_air::{air::CairoProof, prove_cairo, verify_cairo},
    input::{plain::input_from_finished_runner, CairoInput},
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
    pub cairo_input: CairoInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceGenOutputJS {
    execution_resources: ExecutionResources,
    cairo_input: Vec<u8>,
}

#[wasm_bindgen]
pub fn run_trace_gen(program_content_js: JsValue) -> Result<JsValue, JsValue> {
    set_panic_hook();

    let program_content: Vec<u8> = serde_wasm_bindgen::from_value(program_content_js)?;
    let trace_gen_output = trace_gen(program_content)
        .map_err(|e| JsValue::from(format!("Failed to generate trace: {e}")))?;
    Ok(serde_wasm_bindgen::to_value(&TraceGenOutputJS {
        cairo_input: serde_json::to_vec(&trace_gen_output.cairo_input)
            .map_err(|e| JsValue::from(format!("Failed to serialize input: {e}")))?,
        execution_resources: trace_gen_output.execution_resources,
    })?)
}

#[wasm_bindgen]
pub fn run_prove(cairo_input_js: JsValue) -> Result<JsValue, JsValue> {
    set_panic_hook();

    let cairo_input: CairoInput =
        serde_json::from_slice(&serde_wasm_bindgen::from_value::<Vec<u8>>(cairo_input_js)?)
            .map_err(|e| JsValue::from(format!("Failed to deserialize input: {e}")))?;
    let cairo_proof =
        prove(cairo_input).map_err(|e| JsValue::from(format!("Failed to generate proof: {e}")))?;
    Ok(serde_wasm_bindgen::to_value(
        &serde_json::to_vec(&cairo_proof)
            .map_err(|e| JsValue::from(format!("Failed to serialize proof: {e}")))?,
    )?)
}

#[wasm_bindgen]
pub fn run_verify(cairo_proof_js: JsValue) -> Result<JsValue, JsValue> {
    set_panic_hook();

    let cairo_proof: CairoProof<Blake2sMerkleHasher> =
        serde_json::from_slice(&serde_wasm_bindgen::from_value::<Vec<u8>>(cairo_proof_js)?)
            .map_err(|e| JsValue::from(format!("Failed to deserialize proof: {e}")))?;
    let verdict = verify(cairo_proof);
    Ok(serde_wasm_bindgen::to_value(&verdict)?)
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, backend!");
}

pub fn trace_gen(program_content: Vec<u8>) -> Result<TraceGenOutput, VmError> {
    let cairo_run_config = cairo_run::CairoRunConfig {
        trace_enabled: true,
        relocate_mem: true,
        layout: LayoutName::all_cairo,
        proof_mode: true,
        ..Default::default()
    };

    let mut hint_processor = BuiltinHintProcessor::new_empty();
    let cairo_runner_result =
        cairo_run::cairo_run(&program_content, &cairo_run_config, &mut hint_processor);

    let cairo_runner = match cairo_runner_result {
        Ok(runner) => runner,
        Err(error) => {
            eprintln!("{error}");
            return Err(VmError::Runner(error));
        }
    };

    Ok(TraceGenOutput {
        execution_resources: cairo_runner
            .get_execution_resources()
            .map_err(|e| CairoRunError::Runner(e))?,
        cairo_input: input_from_finished_runner(cairo_runner, false),
    })
}

pub fn prove(cairo_input: CairoInput) -> Result<CairoProof<Blake2sMerkleHasher>, ProvingError> {
    prove_cairo::<Blake2sMerkleChannel>(cairo_input, false, false)
}

pub fn verify(cairo_proof: CairoProof<Blake2sMerkleHasher>) -> bool {
    verify_cairo::<Blake2sMerkleChannel>(cairo_proof).is_ok()
}
