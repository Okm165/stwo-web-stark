use cairo_vm::vm::runners::cairo_runner::CairoRunner;
use itertools::Itertools;
use stwo_cairo_adapter::{
    builtins::MemorySegmentAddresses,
    memory::{MemoryBuilder, MemoryConfig, MemoryEntry},
    vm_import::{adapt_to_stwo_input, RelocatedTraceEntry, VmImportError},
    HashMap, ProverInput, PublicSegmentContext,
};

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn adapt_finished_runner(runner: CairoRunner) -> Result<ProverInput, VmImportError> {
    let memory_iter = runner
        .relocated_memory
        .iter()
        .enumerate()
        .filter_map(|(i, v)| {
            v.map(|v| MemoryEntry {
                address: i as u64,
                value: bytemuck::cast(v.to_bytes_le()),
            })
        });

    let public_input = runner.get_air_public_input()?;

    let trace_iter = match runner.relocated_trace {
        Some(ref trace) => trace.iter().map(|t| RelocatedTraceEntry {
            ap: t.ap,
            pc: t.pc,
            fp: t.fp,
        }),
        None => return Err(VmImportError::TraceNotRelocated),
    };

    let memory_segments: &HashMap<&str, MemorySegmentAddresses> = &public_input
        .memory_segments
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                MemorySegmentAddresses {
                    begin_addr: v.begin_addr,
                    stop_ptr: v.stop_ptr,
                },
            )
        })
        .collect();

    let public_memory_addresses = public_input
        .public_memory
        .iter()
        .map(|s| s.address as u32)
        .collect_vec();

    // TODO(spapini): Add output builtin to public memory.
    adapt_to_stwo_input(
        &trace_iter.collect_vec(),
        MemoryBuilder::from_iter(MemoryConfig::default(), memory_iter),
        public_memory_addresses,
        memory_segments,
        PublicSegmentContext::bootloader_context(),
    )
}
