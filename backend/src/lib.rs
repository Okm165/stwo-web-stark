mod utils;

use cairo_lang_runner::{build_hints_dict, casm_run::format_for_panic, Arg, CairoHintProcessor};
use cairo_vm::{
    cairo_run,
    cairo_run::{cairo_run_program, CairoRunConfig},
    types::{layout_name::LayoutName, program::Program, relocatable::MaybeRelocatable},
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor,
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
use num_bigint::BigInt;
use serde_wasm_bindgen::{self, to_value};
use cairo_lang_executable::executable::{EntryPointKind, Executable};
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


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CairoPieDef {
    pub metadata: CairoPieMetadata,
    pub memory: CairoPieMemory,
    pub execution_resources: ExecutionResources,
    pub additional_data: CairoPieAdditionalData,
    pub version: CairoPieVersion,
}

impl From<CairoPie> for CairoPieDef {
    fn from(item: CairoPie) -> Self {
        CairoPieDef {
            metadata: item.metadata,
            memory: item.memory,
            execution_resources: item.execution_resources,
            additional_data: item.additional_data,
            version: item.version,
        }
    }
}

impl From<CairoPieDef> for CairoPie {
    fn from(item: CairoPieDef) -> Self {
        CairoPie {
            metadata: item.metadata,
            memory: item.memory,
            execution_resources: item.execution_resources,
            additional_data: item.additional_data,
            version: item.version,
        }
    }
}



#[wasm_bindgen]
pub fn execute(executable: JsValue, exacutable_args: JsValue) -> JsValue {
    let executable: Executable = serde_wasm_bindgen::from_value(executable).unwrap();

    let data = executable
        .program
        .bytecode
        .iter()
        .map(Felt252::from)
        .map(MaybeRelocatable::from)
        .collect();

    let (hints, string_to_hint) = build_hints_dict(&executable.program.hints);

    let entrypoint = executable
        .entrypoints
        .iter()
        .find(|e| matches!(e.kind, EntryPointKind::Bootloader))
        .with_context(|| "no `Bootloader` entrypoint found")
        .unwrap();

    let program = Program::new(
        entrypoint.builtins.clone(),
        data,
        Some(entrypoint.offset),
        hints,
        Default::default(),
        Default::default(),
        vec![],
        None,
    )
    .with_context(|| "failed setting up program")
    .unwrap();

    let user_args: Vec<BigInt> = serde_wasm_bindgen::from_value(exacutable_args).unwrap();

    let mut hint_processor = CairoHintProcessor {
        runner: None,
        user_args: vec![vec![Arg::Array(
            user_args.iter().map(|x| Arg::Value(x.into())).collect(),
        )]],
        string_to_hint,
        starknet_state: Default::default(),
        run_resources: Default::default(),
        syscalls_used_resources: Default::default(),
        no_temporary_segments: false,
        markers: Default::default(),
        panic_traceback: Default::default(),
    };

    let cairo_run_config = CairoRunConfig {
        allow_missing_builtins: Some(true),
        layout: LayoutName::all_cairo,
        proof_mode: false,
        secure_run: None,
        relocate_mem: true,
        trace_enabled: true,
        ..Default::default()
    };

    let runner = cairo_run_program(&program, &cairo_run_config, &mut hint_processor)
        .map_err(|err| {
            if let Some(panic_data) = hint_processor.markers.last() {
                anyhow!(format_for_panic(panic_data.iter().copied()))
            } else {
                anyhow::Error::from(err).context("Cairo program run failed")
            }
        })
        .unwrap();

    let output_value = runner.get_cairo_pie().unwrap();

    return to_value(&output_value).unwrap();
}

#[wasm_bindgen]
pub fn trace_gen_from_obj(pie: JsValue) -> Result<JsValue, JsValue> {
    let cairo_pie: CairoPieDef = serde_wasm_bindgen::from_value(pie).unwrap();
    let trace_gen_output = trace_gen(cairo_pie.into())
        .map_err(|e| JsValue::from(format!("Failed to generate trace: {e}")))?;
    Ok(serde_wasm_bindgen::to_value(&TraceGenOutputJS {
        prover_input: serde_json::to_string(&trace_gen_output.prover_input)
            .map_err(|e| JsValue::from(format!("Failed to serialize prover input: {e}")))?,
        execution_resources: serde_json::to_string(&trace_gen_output.execution_resources)
            .map_err(|e| JsValue::from(format!("Failed to serialize execution resources: {e}")))?,
    })?)
}

#[wasm_bindgen]
pub fn run_trace_gen(program_content_js: JsValue) -> Result<JsValue, JsValue> {
    set_panic_hook();

    let input: Vec<u8> = serde_wasm_bindgen::from_value(program_content_js)?;
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
        layout: LayoutName::all_cairo,
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
