//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use stwo_web_stark::{from_zip_archive, prove, trace_gen, verify};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn trace_gen_prove_verify() {
    let reader = std::io::Cursor::new(include_bytes!("fibonacci.zip"));
    let zip_archive = zip::ZipArchive::new(reader).unwrap();

    let pie = from_zip_archive(zip_archive).unwrap();
    let trace_gen_output = trace_gen(pie).unwrap();
    let cairo_proof = prove(trace_gen_output.prover_input).unwrap();
    let verdict = verify(cairo_proof);
    assert!(verdict);
}
