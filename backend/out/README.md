# stwo-web-stark

**Wasm package for client-side proving and verification of Cairo CPU traces in web environments.**  
This package includes utilities to execute Cairo programs, generate proofs, and verify them—all within the browser.

---

## ✨ Features

- **Run Cairo Programs**: Execute compiled Cairo programs and generate CPU traces.
- **Generate Proofs**: Create proofs for the generated CPU traces.
- **Verify Proofs**: Verify the proofs in a browser-based environment.

---

## 🚀 Example Usage

### 1. Run a Cairo Program

```typescript
import init, { run_trace_gen } from "stwo-web-stark";

await init(); // Initialize the WASM module
const trace = await run_trace_gen(input); // input: string - compiled Cairo program
console.log(trace); // Cairo runner output
```

### 2. Generate a Proof

```typescript
import init, { run_prove } from "stwo-web-stark";

await init(); // Initialize the WASM module
const proof = await run_prove(trace.prover_input); // Generate proof from CPU trace
console.log(proof); // Outputs the generated proof
```

### 3. Verify the Proof

```typescript
import init, { run_verify } from "stwo-web-stark";

await init(); // Initialize the WASM module
const verdict = await run_verify(proof); // Verify the proof
console.log(verdict); // Outputs true/false for proof validity
```

---

## 🛠️ Development

### Build the WASM Package

To build the WASM package using `wasm-pack`, run:

```bash
wasm-pack build --release --out-dir out --target web
```

### Run Tests in Headless Browsers

To test the package in a headless browser (e.g., Chrome), use:

```bash
wasm-pack test --release --headless --chrome
```