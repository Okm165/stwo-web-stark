//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use cairo_vm::types::program::Program;
use stwo_web_stark::{prove, trace_gen, verify};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn trace_gen_prove_verify() {
    let program = Program::from_bytes(include_bytes!("fibonacci_1000.json"), None).unwrap();
    let trace_gen_output = trace_gen(program).unwrap();
    let cairo_proof = prove(trace_gen_output.prover_input).unwrap();
    let verdict = verify(cairo_proof);
    assert!(verdict);
}
