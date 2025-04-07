
use num_bigint::{BigInt, ToBigInt};
use serde::{Deserialize, Serialize};
use cairo_vm::types::builtin_name::BuiltinName;

use crate::hints::{Hint};

// #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Executable {
    /// The bytecode of the program.
    pub program: AssembledCairoProgram,
    /// The available entrypoints for the program.
    pub entrypoints: Vec<ExecutableEntryPoint>,
}

/// Information about an executable entrypoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableEntryPoint {
    /// The used builtins of the function.
    pub builtins: Vec<BuiltinName>,
    /// The offset of the entrypoint in the bytecode.
    pub offset: usize,
    /// The kind of the entrypoint.
    pub kind: EntryPointKind,
}

// #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssembledCairoProgram {
    /// The bytecode of the program.
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "serialize_big_ints", deserialize_with = "deserialize_big_ints")
    )]
    pub bytecode: Vec<BigInt>,
    /// The list of hints, and the instruction index they refer to.
    pub hints: Vec<(usize, Vec<Hint>)>,
}


/// The kind of an entrypoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryPointKind {
    /// Entrypoint is for running it using a bootloader.
    ///
    /// The entrypoint is a function, ending with a `ret`, expecting the builtins as its parameters.
    Bootloader,
    /// Entrypoint is for running this executable as a standalone program.
    ///
    /// The entrypoint starts with `ap += <builtins.len()>` and expected the builtins to be injected
    /// there, and ends with an infinite loop.
    Standalone,
}