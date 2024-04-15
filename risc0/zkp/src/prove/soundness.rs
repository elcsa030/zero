// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A soundness calculator for the RISC Zero STARK protocol that secures the
//! RISC Zero zkVM. Soundness for STARK protocols can be analyzed under a
//! number of different cryptographic assumptions. RISC Zero targets 100 bits
//! of conjectured soundness, using the Toy Problem Conjecture. For
//! completeness, we also include analysis in three other regimes:
//!
//! - Conjectured soundness using Conjecture 8.4 from [Proximity Gaps] and
//!   Conjecture 2.3 from [DEEP-FRI]
//! - Proven soundness in the list-decoding regime
//! - Proven soundness in the unique-decoding regime

use risc0_core::field::{baby_bear, ExtElem};

use crate::{
    adapter::{REGISTER_GROUP_ACCUM, REGISTER_GROUP_CODE, REGISTER_GROUP_DATA},
    hal::Hal,
    taps::TapSet,
    FRI_FOLD, FRI_MIN_DEGREE, INV_RATE,
};

/// Johnson parameter. See https://eprint.iacr.org/2022/1216
const M: f32 = 16.0;

/// Rate
const RHO: f32 = 1.0 / INV_RATE as f32;

/// η in Conjecture 8.4 of the Proximity Gaps paper
/// [BCIKS21](https://eprint.iacr.org/2020/654.pdf)
const ETA: f32 = 0.05;

/// Compute the security level of the system based on the proven FRI
/// list-decoding regime (up to 1-sqrt(rate)).
pub fn proven<H: Hal>(taps: &TapSet, coeffs_size: usize) -> f32 {
    let params = parameters::<H>(taps, coeffs_size);
    let e_proximity_gap = params.e_proximity_gap_proven();

    // α = (1 + 1/2m) * sqrt(ρ)
    let alpha = (1.0 + 1.0 / (2.0 * M)) * RHO.sqrt();

    let theta = 1.0 - alpha;
    let l_plus = {
        let rho_plus = (params.trace_domain_size + params.biggest_combo) / params.lde_domain_size;
        let m_plus = 1.0 / (params.biggest_combo * (alpha / rho_plus.sqrt() - 1.0));
        let m_plus = m_plus.ceil();

        (m_plus + 0.5) / rho_plus.sqrt()
    };
    soundness_error(&params, theta, e_proximity_gap, l_plus)
}

/// Compute the security level of the system based on the FRI list-decoding
/// conjecture (up to 1-rate).
pub fn conjectured_strict<H: Hal>(taps: &TapSet, coeffs_size: usize) -> f32 {
    let params = parameters::<H>(taps, coeffs_size);
    let theta = 1.0 - RHO - ETA;
    let e_proximity_gap = params.e_proximity_gap_conjectured();
    let l_plus = {
        let rho_plus = (params.trace_domain_size + params.biggest_combo) / params.lde_domain_size;
        let epsilon_plus = 1.0 - rho_plus - theta;
        let c_rho = 1; // unspecified exponent parameter in DEEP-FRI, Conjecture 2.3
        (params.lde_domain_size / epsilon_plus).powi(c_rho)
    };
    soundness_error(&params, theta, e_proximity_gap, l_plus)
}

/// Compute the system security following the Toy Model conjecture of ethSTARK.
/// This conjecture states that:
/// 1. any AIR is as secure as the "simplest AIR" (1 column and degree 1
///    constraint).
/// 2. The security of FRI matches its known upper bound (rather than the proven
///    lower bound).
pub fn toy_model_security<H: Hal>() -> f32 {
    let ext_size = H::ExtElem::EXT_SIZE as f32;
    let field_size = baby_bear::P as f32;
    let ext_field_size = field_size.powf(ext_size);

    let constraints_error = 1f32 / ext_field_size;
    let fri_error = RHO.powi(crate::QUERIES as i32);

    let sum = constraints_error + fri_error;
    sum.log2().abs()
}

/// Helper function. Combines the soundness error terms from different system components.
fn soundness_error(params: &Params, theta: f32, e_proximity_gap: f32, l_plus: f32) -> f32 {
    let plonk_plookup_error = params.plonk_plookup_error();
    let fri_error = params.e_fri(theta, e_proximity_gap);
    let deep_ali_error = params.e_deep_ali(l_plus);
    let sum = plonk_plookup_error + fri_error + deep_ali_error;
    sum.log2().abs()
}

/// (1 - θ)^QUERIES
fn e_fri_queries(theta: f32) -> f32 {
    (1.0 - theta).powi(crate::QUERIES as i32)
}

/// Compute the number of folding rounds
fn num_folding_rounds(coeffs_size: usize, ext_size: usize) -> usize {
    let mut num_folding_rounds = 0;
    let mut coeffs_size = coeffs_size;
    while coeffs_size / ext_size > FRI_MIN_DEGREE {
        coeffs_size /= FRI_FOLD;
        num_folding_rounds += 1;
    }
    num_folding_rounds
}

#[derive(Copy, Clone)]
struct Params {
    /// The number of duplicated data columns in the witness that appear
    /// due to the permutation from trace_time to trace_mem
    n_sigma_mem: usize,
    /// The number of columns in the witness that appear due to the bytes-lookup
    n_sigma_bytes: usize,
    /// No. of trace polynomials
    n_trace_polys: f32,
    /// Max degree of the constraint system, i.e. no. of segment polynomials
    d: f32,
    /// Max. no. of entries used from a single column
    biggest_combo: f32,
    /// Field extension degree
    ext_size: usize,
    /// Size of extension field
    ext_field_size: f32,
    /// Domain size of the trace
    trace_domain_size: f32,
    /// Domain size after low-degree extension
    lde_domain_size: f32,
    /// Number of folding rounds in FRI
    num_folding_rounds: usize,
}

/// Compute circuit parameters given a tapset, number of trace rows and all the
/// global constants.
fn parameters<H: Hal>(taps: &TapSet, coeffs_size: usize) -> Params {
    // Circuit-specific info
    // FIXME: get from circuit instead of hard-coding
    let n_sigma_mem = 5;
    // FIXME: get from circuit instead of hard-coding
    let n_sigma_bytes = 15;
    let n_trace_polys = {
        let w_accum = taps.group_size(REGISTER_GROUP_ACCUM) as f32;
        let w_code = taps.group_size(REGISTER_GROUP_CODE) as f32;
        let w_data = taps.group_size(REGISTER_GROUP_DATA) as f32;

        w_accum + w_code + w_data
    };
    // Max degree of the constraint system, i.e. no. of segment polys
    // FIXME: get from circuit instead of hard-coding
    let d = 4.0;

    let biggest_combo = taps.combos().map(|combo| combo.size()).max().unwrap() as f32;

    let ext_size = H::ExtElem::EXT_SIZE;
    let field_size = baby_bear::P as f32;
    let ext_field_size = field_size.powf(ext_size as f32);
    let trace_domain_size = (coeffs_size / ext_size) as f32;
    let lde_domain_size = trace_domain_size * INV_RATE as f32;

    let num_folding_rounds = num_folding_rounds(coeffs_size, ext_size);

    Params {
        n_sigma_mem,
        n_sigma_bytes,
        n_trace_polys,
        d,
        biggest_combo,
        ext_size,
        ext_field_size,
        trace_domain_size,
        lde_domain_size,
        num_folding_rounds,
    }
}

impl Params {
    fn plonk_plookup_error(&self) -> f32 {
        let n_columns = self.n_sigma_mem + self.n_sigma_bytes;
        self.ext_size as f32 * n_columns as f32 * self.trace_domain_size / self.ext_field_size
    }

    /// (m + 1/2)^7 / (3 * sqrt(ρ)^3) * |D|^2 / |K|
    fn e_proximity_gap_proven(&self) -> f32 {
        (M + 0.5).powi(7) / (3.0 * RHO.sqrt().powi(3))
            * (self.lde_domain_size.powi(2) / self.ext_field_size)
    }

    /// Conjecture 8.4 [BCIKS21]
    fn e_proximity_gap_conjectured(&self) -> f32 {
        let c_1 = 1; // first parameter in Proximity Gaps, Conjecture 8.4
        let c_2 = 1; // second parameter in Proximity Gaps, Conjecture 8.4

        // 1 / (ηρ)^c_1
        let first_term = 1.0 / (ETA * RHO).powi(c_1);

        //   (l • n)^c_2 / q
        // = (n_trace_polys • |D|)^c_2 / ext_field_size
        let second_term =
            (self.n_trace_polys * self.lde_domain_size).powi(c_2) / self.ext_field_size;

        first_term * second_term
    }

    fn e_fri_constant(&self, e_proximity_gap: f32) -> f32 {
        // (w_rap + d - 1/2) * e_proximity_gap
        let first_term = (self.n_trace_polys + self.d - 0.5) * e_proximity_gap;

        // (2m + 1) * (|D| + 1) * FRI_FOLD * num_folding_rounds
        // ----------------------------------------------------
        //                 sqrt(ρ) * |K|
        let second_term = {
            let numerator = (2.0 * M + 1.0)
                * (self.lde_domain_size + 1.0)
                * (FRI_FOLD * self.num_folding_rounds) as f32;
            let denominator = RHO.sqrt() * self.ext_field_size;
            numerator / denominator
        };

        first_term + second_term
    }

    fn e_fri(&self, theta: f32, e_proximity_gap: f32) -> f32 {
        let e_fri_constant = self.e_fri_constant(e_proximity_gap);

        let e_fri_queries = e_fri_queries(theta);

        e_fri_constant + e_fri_queries
    }

    fn e_ali(&self, l_plus: f32) -> f32 {
        l_plus * self.n_trace_polys / self.ext_field_size
    }

    fn e_deep(&self, l_plus: f32) -> f32 {
        let h_plus = self.trace_domain_size + self.biggest_combo;

        let numerator = self.d * (h_plus - 1.0) + (self.trace_domain_size - 1.0);
        let denominator = self.ext_field_size - self.trace_domain_size - self.lde_domain_size;
        l_plus * numerator / denominator
    }

    fn e_deep_ali(&self, l_plus: f32) -> f32 {
        self.e_deep(l_plus) + self.e_ali(l_plus)
    }
}

#[cfg(test)]
mod tests {
    use risc0_core::field::baby_bear::BabyBear;

    use crate::hal::cpu::CpuHal;

    #[test]
    fn toy_model() {
        let security = super::toy_model_security::<CpuHal<BabyBear>>();
        assert_eq!(security, 100.0);
    }
}
