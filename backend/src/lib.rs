mod error;
mod executable;
mod hint_processor;
mod hints;
mod runner;
mod utils;

use std::convert::TryFrom;

use anyhow::{anyhow, bail, ensure, Context, Error, Result};
use cairo_lang_sierra::{
    extensions::circuit,
    program::{Function, GenericArg, Program as SierraProgram},
};
use cairo_vm::{
    cairo_run::{self, CairoRunConfig},
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor,
    types::{
        layout,
        layout_name::LayoutName,
        program::Program,
        relocatable::{MaybeRelocatable, Relocatable},
    },
    vm::{
        errors::cairo_run_errors::CairoRunError,
        runners::{
            cairo_pie::{
                CairoPie, CairoPieAdditionalData, CairoPieMemory, CairoPieMetadata, CairoPieVersion,
            },
            cairo_runner::{ExecutionResources, RunResources},
        },
    },
    Felt252,
};
use executable::{EntryPointKind, Executable};
use num_bigint::BigInt;
use runner::{cairo_run_program, Cairo1RunConfig, FuncArg};
use serde::de::DeserializeOwned;
// use crate::runner::{build_hints_dict, format_for_panic, Arg, CairoHintProcessor};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
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

#[derive(Debug, Clone)]
pub enum Arg {
    Value(Felt252),
    Array(Vec<Arg>),
}

impl Arg {
    /// Returns the size of the argument in the vm.
    pub fn size(&self) -> usize {
        match self {
            Self::Value(_) => 1,
            Self::Array(_) => 2,
        }
    }
}
impl From<Felt252> for Arg {
    fn from(value: Felt252) -> Self {
        Self::Value(value)
    }
}

#[derive(Debug, Clone, Default)]
struct FuncArgs(Vec<FuncArg>);

/// Processes an iterator of format [s1, s2,.., sn, "]", ...], stopping at the first "]" string
/// and returning the array [f1, f2,.., fn] where fi = Felt::from_dec_str(si)
fn process_array<'a>(iter: &mut impl Iterator<Item = &'a str>) -> Result<FuncArg, String> {
    let mut array = vec![];
    for value in iter {
        match value {
            "]" => break,
            _ => array.push(
                Felt252::from_dec_str(value)
                    .map_err(|_| format!("\"{}\" is not a valid felt", value))?,
            ),
        }
    }
    Ok(FuncArg::Array(array))
}

/// Parses a string of ascii whitespace separated values, containing either numbers or series of
/// numbers wrapped in brackets Returns an array of felts and felt arrays
fn process_args(value: &str) -> Result<FuncArgs, String> {
    let mut args = Vec::new();
    // Split input string into numbers and array delimiters
    let mut input = value.split_ascii_whitespace().flat_map(|mut x| {
        // We don't have a way to split and keep the separate delimiters so we do it manually
        let mut res = vec![];
        if let Some(val) = x.strip_prefix('[') {
            res.push("[");
            x = val;
        }
        if let Some(val) = x.strip_suffix(']') {
            if !val.is_empty() {
                res.push(val)
            }
            res.push("]")
        } else if !x.is_empty() {
            res.push(x)
        }
        res
    });
    // Process iterator of numbers & array delimiters
    while let Some(value) = input.next() {
        match value {
            "[" => args.push(process_array(&mut input)?),
            _ => args.push(FuncArg::Single(
                Felt252::from_dec_str(value)
                    .map_err(|_| format!("\"{}\" is not a valid felt", value))?,
            )),
        }
    }
    Ok(FuncArgs(args))
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
pub fn run_execute_trace_gen(program: JsValue, args: JsValue) -> JsValue {
    let input: String = serde_wasm_bindgen::from_value(args).unwrap();

    let sierra_program: SierraProgram = serde_wasm_bindgen::from_value(program).unwrap();
    
    let trace = execute_trace_gen(sierra_program, input);

    return to_value(&trace).unwrap();
}

fn execute_trace_gen(sierra_program: SierraProgram, args: String) -> TraceGenOutputJS{
    let user_args = process_args(&args)
        .map_err(|e| JsValue::from(format!("Failed to process args: {e}")))
        .unwrap();
    let a: &[FuncArg] = &user_args.0;

    let cairo_run_config = Cairo1RunConfig {
        proof_mode: false,
        serialize_output: false,
        relocate_mem: true,
        layout: LayoutName::all_cairo,
        trace_enabled: true,
        args: a,
        finalize_builtins: true,
        append_return_values: false,
        ..Default::default()
    };

    let runner = cairo_run_program(&sierra_program, cairo_run_config).0;

    let output_value = runner.get_cairo_pie().unwrap();

    let trace_gen_output = trace_gen(output_value)
        .map_err(|e| JsValue::from(format!("Failed to generate trace: {e}")))
        .unwrap();
    return TraceGenOutputJS {
        prover_input: serde_json::to_string(&trace_gen_output.prover_input)
            .map_err(|e| JsValue::from(format!("Failed to serialize prover input: {e}")))
            .unwrap(),
        execution_resources: serde_json::to_string(&trace_gen_output.execution_resources)
            .map_err(|e| JsValue::from(format!("Failed to serialize execution resources: {e}")))
            .unwrap(),
    }
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

#[cfg(test)]
mod tests {
    // Import the outer function into the test module's scope
    use super::*;

    #[test]
    fn test_execute() {
        let input: String = "5".to_string();

        // let user_args = process_args(&input).unwrap();

        let sierra_program: SierraProgram = match read_json_file(
            "/home/esdras/Downloads/stwo-web-stark/backend/swstest.sierra.json",
        ) {
            Ok(program) => program,
            Err(e) => {
                panic!("Failed to read Sierra program: {}", e);
            }
        };

        let trace = execute_trace_gen(sierra_program, input);

        // let a: &[FuncArg] = &user_args.0;

        // let cairo_run_config = Cairo1RunConfig {
        //     proof_mode: false,
        //     serialize_output: false,
        //     relocate_mem: true,
        //     layout: LayoutName::all_cairo,
        //     trace_enabled: true,
        //     args: a,
        //     finalize_builtins: true,
        //     append_return_values: false,

        //     ..Default::default()
        // };

        // let runner = cairo_run_program(&sierra_program, cairo_run_config).0;

        // let output_value = runner.get_cairo_pie().unwrap();
        // let cairo_pie: CairoPieDef = output_value.into();
        // let trace_gen_output = trace_gen(cairo_pie.into())
        //     .map_err(|e| JsValue::from(format!("Failed to generate trace: {e}")))
        //     .unwrap();
        // let tg = &TraceGenOutputJS {
        //     prover_input: serde_json::to_string(&trace_gen_output.prover_input)
        //         .map_err(|e| JsValue::from(format!("Failed to serialize prover input: {e}")))
        //         .unwrap(),
        //     execution_resources: serde_json::to_string(&trace_gen_output.execution_resources)
        //         .map_err(|e| JsValue::from(format!("Failed to serialize execution resources: {e}")))
        //         .unwrap(),
        // };

        let prover_input: ProverInput = serde_json::from_str(trace.prover_input.as_str())
            .map_err(|e| JsValue::from(format!("Failed to deserialize prover input: {e}")))
            .unwrap();
        let proof = prove(prover_input)
            .map_err(|e| JsValue::from(format!("Failed to generate proof: {e}")))
            .unwrap();
        let sp = serde_json::to_string(&proof).unwrap();
        let proof: CairoProof<Blake2sMerkleHasher> = serde_json::from_str(sp.as_str())
            .map_err(|e| JsValue::from(format!("Failed to deserialize proof: {e}")))
            .unwrap();
        let verdict = verify(proof);
        assert!(verdict);
    }

    fn read_json_file(file_path: &str) -> Result<SierraProgram> {
        let file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);

        // Deserialize the JSON content into the Person struct.
        let program: SierraProgram = serde_json::from_reader(reader)?;
        Ok(program)
    }
}
