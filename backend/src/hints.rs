use num_bigint::{BigInt, ToBigInt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
pub enum CoreHint {
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 0))]
    AllocSegment { dst: CellRef },
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 1))]
    TestLessThan { lhs: ResOperand, rhs: ResOperand, dst: CellRef },
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 2))]
    TestLessThanOrEqual { lhs: ResOperand, rhs: ResOperand, dst: CellRef },
    /// Variant of TestLessThanOrEqual that compares addresses.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 28))]
    TestLessThanOrEqualAddress { lhs: ResOperand, rhs: ResOperand, dst: CellRef },
    /// Multiplies two 128-bit integers and returns two 128-bit integers: the high and low parts of
    /// the product.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 3))]
    WideMul128 { lhs: ResOperand, rhs: ResOperand, high: CellRef, low: CellRef },
    /// Computes lhs/rhs and returns the quotient and remainder.
    ///
    /// Note: the hint may be used to write an already assigned memory cell.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 4))]
    DivMod { lhs: ResOperand, rhs: ResOperand, quotient: CellRef, remainder: CellRef },
    /// Divides dividend (represented by 2 128bit limbs) by divisor (represented by 2 128bit
    /// limbs). Returns the quotient (represented by 2 128bit limbs) and remainder (represented by
    /// 2 128bit limbs). In all cases - `name`0 is the least significant limb.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 5))]
    Uint256DivMod {
        dividend0: ResOperand,
        dividend1: ResOperand,
        divisor0: ResOperand,
        divisor1: ResOperand,
        quotient0: CellRef,
        quotient1: CellRef,
        remainder0: CellRef,
        remainder1: CellRef,
    },
    /// Divides dividend (represented by 4 128bit limbs) by divisor (represented by 2 128bit
    /// limbs). Returns the quotient (represented by 4 128bit limbs) and remainder (represented
    /// by 2 128bit limbs).
    /// In all cases - `name`0 is the least significant limb.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 6))]
    Uint512DivModByUint256 {
        dividend0: ResOperand,
        dividend1: ResOperand,
        dividend2: ResOperand,
        dividend3: ResOperand,
        divisor0: ResOperand,
        divisor1: ResOperand,
        quotient0: CellRef,
        quotient1: CellRef,
        quotient2: CellRef,
        quotient3: CellRef,
        remainder0: CellRef,
        remainder1: CellRef,
    },
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 7))]
    SquareRoot { value: ResOperand, dst: CellRef },
    /// Computes the square root of value_low<<128+value_high, stores the 64bit limbs of the result
    /// in sqrt0 and sqrt1 as well as the 128bit limbs of the remainder in remainder_low and
    /// remainder_high. The remainder is defined as `value - sqrt**2`.
    /// Lastly it checks whether `2*sqrt - remainder >= 2**128`.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 8))]
    Uint256SquareRoot {
        value_low: ResOperand,
        value_high: ResOperand,
        sqrt0: CellRef,
        sqrt1: CellRef,
        remainder_low: CellRef,
        remainder_high: CellRef,
        sqrt_mul_2_minus_remainder_ge_u128: CellRef,
    },
    /// Finds some `x` and `y` such that `x * scalar + y = value` and `x <= max_x`.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 9))]
    LinearSplit { value: ResOperand, scalar: ResOperand, max_x: ResOperand, x: CellRef, y: CellRef },
    /// Allocates a new dict segment, and write its start address into the dict_infos segment.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 10))]
    AllocFelt252Dict { segment_arena_ptr: ResOperand },
    /// Fetch the previous value of a key in a dict, and write it in a new dict access.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 11))]
    Felt252DictEntryInit { dict_ptr: ResOperand, key: ResOperand },
    /// Similar to Felt252DictWrite, but updates an existing entry and does not write the previous
    /// value to the stack.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 12))]
    Felt252DictEntryUpdate { dict_ptr: ResOperand, value: ResOperand },
    /// Retrieves the index of the given dict in the dict_infos segment.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 13))]
    GetSegmentArenaIndex { dict_end_ptr: ResOperand, dict_index: CellRef },
    /// Initialized the lists of accesses of each key of a dict as a preparation of squash_dict.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 14))]
    InitSquashData {
        dict_accesses: ResOperand,
        ptr_diff: ResOperand,
        n_accesses: ResOperand,
        big_keys: CellRef,
        first_key: CellRef,
    },
    /// Retrieves the current index of a dict access to process.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 15))]
    GetCurrentAccessIndex { range_check_ptr: ResOperand },
    /// Writes if the squash_dict loop should be skipped.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 16))]
    ShouldSkipSquashLoop { should_skip_loop: CellRef },
    /// Writes the delta from the current access index to the next one.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 17))]
    GetCurrentAccessDelta { index_delta_minus1: CellRef },
    /// Writes if the squash_dict loop should be continued.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 18))]
    ShouldContinueSquashLoop { should_continue: CellRef },
    /// Writes the next dict key to process.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 19))]
    GetNextDictKey { next_key: CellRef },
    /// Finds the two small arcs from within [(0,a),(a,b),(b,PRIME)] and writes it to the
    /// range_check segment.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 20))]
    AssertLeFindSmallArcs { range_check_ptr: ResOperand, a: ResOperand, b: ResOperand },
    /// Writes if the arc (0,a) was excluded.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 21))]
    AssertLeIsFirstArcExcluded { skip_exclude_a_flag: CellRef },
    /// Writes if the arc (a,b) was excluded.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 22))]
    AssertLeIsSecondArcExcluded { skip_exclude_b_minus_a: CellRef },
    /// Samples a random point on the EC.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 23))]
    RandomEcPoint { x: CellRef, y: CellRef },
    /// Computes the square root of `val`, if `val` is a quadratic residue, and of `3 * val`
    /// otherwise.
    ///
    /// Since 3 is not a quadratic residue, exactly one of `val` and `3 * val` is a quadratic
    /// residue (unless `val` is 0). This allows proving that `val` is not a quadratic residue.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 24))]
    FieldSqrt { val: ResOperand, sqrt: CellRef },
    /// Prints the values from start to end.
    /// Both must be pointers.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 25))]
    DebugPrint { start: ResOperand, end: ResOperand },
    /// Returns an address with `size` free locations afterwards.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 26))]
    AllocConstantSize { size: ResOperand, dst: CellRef },
    /// Provides the inverse of b (represented by 2 128-bit limbs) modulo n (represented by 2
    /// 128-bit limbs), or a proof that b has no inverse.
    ///
    /// In case b has an inverse: Returns `r` and `k` such that:
    ///   * `r = 1 / b (mod n)`
    ///   * `k = (r * b - 1) / n`
    ///   * `g0_or_no_inv = 0`
    ///
    /// In case b has no inverse: Returns `g`, `s`, and `t`, such that:
    /// `g > 1`
    /// `g == 2 || g % 2 == 1` (in particular, `g0_or_no_inv = g0 != 0`)
    /// `g * s = b`
    /// `g * t = n`
    ///
    /// The case `n == 1` is considered "no-inverse" (special case).
    /// In this case: Returns `g == 1`, `s == b` and `t == 1`.
    /// All no-inverse requirements are satisfied, except for `g > 1`.
    ///
    /// In all cases - `name`0 is the least significant limb.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 27))]
    U256InvModN {
        b0: ResOperand,
        b1: ResOperand,
        n0: ResOperand,
        n1: ResOperand,
        g0_or_no_inv: CellRef,
        g1_option: CellRef,
        s_or_r0: CellRef,
        s_or_r1: CellRef,
        t_or_k0: CellRef,
        t_or_k1: CellRef,
    },

    #[cfg_attr(feature = "parity-scale-codec", codec(index = 29))]
    EvalCircuit {
        n_add_mods: ResOperand,
        add_mod_builtin: ResOperand,
        n_mul_mods: ResOperand,
        mul_mod_builtin: ResOperand,
    },
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
pub enum Hint {
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 0))]
    Core(CoreHintBase),
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 1))]
    Starknet(StarknetHint),
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 2))]
    #[cfg_attr(feature = "schemars", schemars(skip))]
    External(ExternalHint),
}

impl Hint {
    pub fn representing_string(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
pub enum ExternalHint {
    /// Relocates a segment from `src` to `dst`.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 0))]
    AddRelocationRule { src: ResOperand, dst: ResOperand },
    /// Writes a run argument of number `index` to `dst` and on.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 1))]
    WriteRunParam { index: ResOperand, dst: CellRef },
    /// Stores an array marker in the HintProcessor. Useful for debugging.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 2))]
    AddMarker { start: ResOperand, end: ResOperand },
    /// Adds a trace call with the given flag to the HintProcessor. Useful for debugging.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 3))]
    AddTrace { flag: ResOperand },
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
pub enum StarknetHint {
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 0))]
    SystemCall { system: ResOperand },
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 1))]
    #[cfg_attr(feature = "schemars", schemars(skip))]
    Cheatcode {
        selector: BigIntAsHex,
        input_start: ResOperand,
        input_end: ResOperand,
        output_start: CellRef,
        output_end: CellRef,
    },
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
pub enum CoreHintBase {
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 0))]
    Core(CoreHint),
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 1))]
    Deprecated(DeprecatedHint),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
pub struct CellRef {
    pub register: Register,
    pub offset: i16,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
pub enum ResOperand {
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 0))]
    Deref(CellRef),
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 1))]
    DoubleDeref(CellRef, i16),
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 2))]
    Immediate(BigIntAsHex),
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 3))]
    BinOp(BinOpOperand),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
pub enum Register {
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 0))]
    AP,
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 1))]
    FP,
}

#[derive(Default, Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct BigIntAsHex {
    /// A field element that encodes the signature of the called function.
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "serialize_big_int", deserialize_with = "deserialize_big_int")
    )]
    #[cfg_attr(feature = "schemars", schemars(schema_with = "big_int_schema"))]
    pub value: BigInt,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
pub struct BinOpOperand {
    pub op: Operation,
    pub a: CellRef,
    pub b: DerefOrImmediate,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
pub enum Operation {
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 0))]
    Add,
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 1))]
    Mul,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
pub enum DerefOrImmediate {
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 0))]
    Deref(CellRef),
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 1))]
    Immediate(BigIntAsHex),
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
pub enum DeprecatedHint {
    /// Asserts that the current access indices list is empty (after the loop).
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 0))]
    AssertCurrentAccessIndicesIsEmpty,
    /// Asserts that the number of used accesses is equal to the length of the original accesses
    /// list.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 1))]
    AssertAllAccessesUsed { n_used_accesses: CellRef },
    /// Asserts that the keys list is empty.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 2))]
    AssertAllKeysUsed,
    /// Asserts that the arc (b, PRIME) was excluded.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 3))]
    AssertLeAssertThirdArcExcluded,
    /// Asserts that the input represents integers and that a<b.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 4))]
    AssertLtAssertValidInput { a: ResOperand, b: ResOperand },
    /// Retrieves and writes the value corresponding to the given dict and key from the vm
    /// dict_manager.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 5))]
    Felt252DictRead { dict_ptr: ResOperand, key: ResOperand, value_dst: CellRef },
    /// Sets the value corresponding to the key in the vm dict_manager.
    #[cfg_attr(feature = "parity-scale-codec", codec(index = 6))]
    Felt252DictWrite { dict_ptr: ResOperand, key: ResOperand, value: ResOperand },
}