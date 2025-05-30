use cairo_lang_sierra::{ids::ConcreteTypeId, program_registry::ProgramRegistryError};
use cairo_lang_sierra_to_casm::{compiler::CompilationError, metadata::MetadataError};
use cairo_vm::{
    air_public_input::PublicInputError,
    cairo_run::EncodeTraceError,
    types::errors::program_errors::ProgramError,
    vm::errors::{
        memory_errors::MemoryError, runner_errors::RunnerError, trace_errors::TraceError,
        vm_errors::VirtualMachineError,
    },
    Felt252,
};
use thiserror::Error;
use std::fmt;

#[derive(Debug)]
pub struct EncodeTraceErrorWrapper(EncodeTraceError);

impl fmt::Display for EncodeTraceErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::error::Error for EncodeTraceErrorWrapper {}

#[derive(Debug)]
pub struct VirtualMachineErrorWrapper(VirtualMachineError);

impl fmt::Display for VirtualMachineErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::error::Error for VirtualMachineErrorWrapper {}

impl VirtualMachineErrorWrapper {
    // Method to access a reference to the inner EncodeTraceError
    pub fn inner(&self) -> &VirtualMachineError {
        &self.0
    }
}

#[derive(Debug)]
struct TraceErrorWrapper(TraceError);

impl fmt::Display for TraceErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::error::Error for TraceErrorWrapper {}

#[derive(Debug)]
struct PublicInputErrorWrapper(PublicInputError);

impl fmt::Display for PublicInputErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::error::Error for PublicInputErrorWrapper {}

#[derive(Debug)]
struct RunnerErrorWrapper(RunnerError);

impl fmt::Display for RunnerErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::error::Error for RunnerErrorWrapper {}

#[derive(Debug)]
struct ProgramErrorWrapper(ProgramError);

impl fmt::Display for ProgramErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::error::Error for ProgramErrorWrapper {}


#[derive(Debug)]
struct MemoryErrorWrapper(MemoryError);

impl fmt::Display for MemoryErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::error::Error for MemoryErrorWrapper {}


#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid arguments")]
    Cli(#[from] clap::Error),
    #[error("Failed to interact with the file system")]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    EncodeTrace(#[from] EncodeTraceErrorWrapper),
    #[error(transparent)]
    VirtualMachine(#[from] VirtualMachineErrorWrapper),
    #[error(transparent)]
    Trace(#[from] TraceErrorWrapper),
    #[error(transparent)]
    PublicInput(#[from] PublicInputErrorWrapper),
    #[error(transparent)]
    Runner(#[from] RunnerErrorWrapper),
    #[error(transparent)]
    ProgramRegistry(#[from] Box<ProgramRegistryError>),
    #[error(transparent)]
    Compilation(#[from] Box<CompilationError>),
    #[error("Failed to compile to sierra:\n {0}")]
    SierraCompilation(String),
    #[error(transparent)]
    Metadata(#[from] MetadataError),
    #[error(transparent)]
    Program(#[from] ProgramErrorWrapper),
    #[error(transparent)]
    Memory(#[from] MemoryErrorWrapper),
    #[error("Program panicked with {0:?}")]
    RunPanic(Vec<Felt252>),
    #[error("Function signature has no return types")]
    NoRetTypesInSignature,
    #[error("No size for concrete type id: {0}")]
    NoTypeSizeForId(ConcreteTypeId),
    #[error("Concrete type id has no debug name: {0}")]
    TypeIdNoDebugName(ConcreteTypeId),
    #[error("No info in sierra program registry for concrete type id: {0}")]
    NoInfoForType(ConcreteTypeId),
    #[error("Failed to extract return values from VM")]
    FailedToExtractReturnValues,
    #[error("Function expects arguments of size {expected} and received {actual} instead.")]
    ArgumentsSizeMismatch { expected: i16, actual: i16 },
    #[error("Function param {param_index} only partially contains argument {arg_index}.")]
    ArgumentUnaligned {
        param_index: usize,
        arg_index: usize,
    },
    #[error("Only programs returning `Array<Felt252>` can be currently proven. Try serializing the final values before returning them")]
    IlegalReturnValue,
    #[error("Only programs with `Array<Felt252>` as an input can be currently proven. Try inputing the serialized version of the input and deserializing it on main")]
    IlegalInputValue,
}