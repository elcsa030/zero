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

use std::rc::Rc;

use cust::{memory::GpuBuffer as _, prelude::*};
use risc0_zkp::{
    core::log2_ceil,
    field::{
        baby_bear::{BabyBearElem, BabyBearExtElem},
        map_pow, Elem, ExtElem, RootsOfUnity,
    },
    hal::{
        cuda::{
            prefix_products, BufferImpl as CudaBuffer, CudaHal, CudaHash, CudaHashPoseidon,
            CudaHashPoseidon2, CudaHashSha256, DeviceExtElem,
        },
        Buffer, CircuitHal,
    },
    INV_RATE, ZK_CYCLES,
};

use crate::{
    GLOBAL_MIX, GLOBAL_OUT, REGISTER_GROUP_ACCUM, REGISTER_GROUP_CODE, REGISTER_GROUP_DATA,
};

const EVAL_FATBIN: &[u8] = include_bytes!(env!("RECURSION_CUDA_EVAL_PATH"));
const STEPS_FATBIN: &[u8] = include_bytes!(env!("RECURSION_CUDA_STEPS_PATH"));

pub struct CudaCircuitHal<CH: CudaHash> {
    hal: Rc<CudaHal<CH>>, // retain a reference to ensure the context remains valid
    eval_module: Module,
    steps_module: Module,
}

impl<CH: CudaHash> CudaCircuitHal<CH> {
    #[tracing::instrument(name = "CudaCircuitHal::new", skip_all)]
    pub fn new(hal: Rc<CudaHal<CH>>) -> Self {
        let eval_module = Module::from_fatbin(EVAL_FATBIN, &[]).unwrap();
        let steps_module = Module::from_fatbin(STEPS_FATBIN, &[]).unwrap();
        Self {
            hal,
            eval_module,
            steps_module,
        }
    }
}

impl<'a, CH: CudaHash> CircuitHal<CudaHal<CH>> for CudaCircuitHal<CH> {
    #[tracing::instrument(skip_all)]
    fn eval_check(
        &self,
        check: &CudaBuffer<BabyBearElem>,
        groups: &[&CudaBuffer<BabyBearElem>],
        globals: &[&CudaBuffer<BabyBearElem>],
        poly_mix: BabyBearExtElem,
        po2: usize,
        steps: usize,
    ) {
        let code = groups[REGISTER_GROUP_CODE];
        let data = groups[REGISTER_GROUP_DATA];
        let accum = groups[REGISTER_GROUP_ACCUM];
        let mix = globals[GLOBAL_MIX];
        let out = globals[GLOBAL_OUT];
        tracing::debug!(
            "check: {}, code: {}, data: {}, accum: {}, mix: {} out: {}",
            check.size(),
            code.size(),
            data.size(),
            accum.size(),
            mix.size(),
            out.size()
        );
        tracing::debug!(
            "total: {}",
            (check.size() + code.size() + data.size() + accum.size() + mix.size() + out.size()) * 4
        );

        const EXP_PO2: usize = log2_ceil(INV_RATE);
        let domain = steps * INV_RATE;
        let rou = BabyBearElem::ROU_FWD[po2 + EXP_PO2];

        let rou = CudaBuffer::copy_from("rou", &[rou]);
        let po2 = CudaBuffer::copy_from("po2", &[po2 as u32]);
        let size = CudaBuffer::copy_from("size", &[domain as u32]);

        let poly_mix_pows = map_pow(poly_mix, crate::info::POLY_MIX_POWERS);
        let poly_mix_pows: &[u32; BabyBearExtElem::EXT_SIZE * crate::info::NUM_POLY_MIX_POWERS] =
            BabyBearExtElem::as_u32_slice(poly_mix_pows.as_slice())
                .try_into()
                .unwrap();

        let mix_pows_name = std::ffi::CString::new("poly_mix").unwrap();
        self.eval_module
            .get_global(&mix_pows_name)
            .unwrap()
            .copy_from(poly_mix_pows)
            .unwrap();

        let kernel = self.eval_module.get_function("eval_check").unwrap();
        let params = self.hal.compute_simple_params(domain);
        unsafe {
            let stream = &self.hal.stream;
            launch!(kernel<<<params.0, params.1, 0, stream>>>(
                check.as_device_ptr(),
                code.as_device_ptr(),
                data.as_device_ptr(),
                accum.as_device_ptr(),
                mix.as_device_ptr(),
                out.as_device_ptr(),
                rou.as_device_ptr(),
                po2.as_device_ptr(),
                size.as_device_ptr()
            ))
            .unwrap();
        }
        self.hal.stream.synchronize().unwrap();
    }

    fn accumulate(
        &self,
        ctrl: &CudaBuffer<BabyBearElem>,
        io: &CudaBuffer<BabyBearElem>,
        data: &CudaBuffer<BabyBearElem>,
        mix: &CudaBuffer<BabyBearElem>,
        accum: &CudaBuffer<BabyBearElem>,
        steps: usize,
    ) {
        let count = steps - ZK_CYCLES;
        let params = self.hal.compute_simple_params(count);

        let wom = vec![DeviceExtElem(BabyBearExtElem::ONE); steps];
        let mut wom = UnifiedBuffer::from_slice(&wom).unwrap();

        tracing::info_span!("step_compute_accum").in_scope(|| {
            let kernel = self
                .steps_module
                .get_function("step_compute_accum")
                .unwrap();
            unsafe {
                let stream = &self.hal.stream;
                launch!(kernel<<<params.0, params.1, 0, stream>>>(
                    ctrl.as_device_ptr(),
                    data.as_device_ptr(),
                    mix.as_device_ptr(),
                    wom.as_device_ptr(),
                    steps as u32,
                    count as u32,
                ))
                .unwrap();
            }
            self.hal.stream.synchronize().unwrap();
        });

        tracing::info_span!("prefix_products").in_scope(|| {
            prefix_products(&mut wom);
        });

        tracing::info_span!("step_verify_accum").in_scope(|| {
            let kernel = self.steps_module.get_function("step_verify_accum").unwrap();
            unsafe {
                let stream = &self.hal.stream;
                launch!(kernel<<<params.0, params.1, 0, stream>>>(
                    ctrl.as_device_ptr(),
                    data.as_device_ptr(),
                    mix.as_device_ptr(),
                    wom.as_device_ptr(),
                    accum.as_device_ptr(),
                    steps as u32,
                    count as u32,
                ))
                .unwrap();
            }
            self.hal.stream.synchronize().unwrap();
        });

        tracing::info_span!("zeroize").in_scope(|| {
            let kernel = self.hal.module.get_function("eltwise_zeroize_fp").unwrap();

            let params = self.hal.compute_simple_params(accum.size());
            unsafe {
                let stream = &self.hal.stream;
                launch!(kernel<<<params.0, params.1, 0, stream>>>(accum.as_device_ptr())).unwrap();
            }

            let params = self.hal.compute_simple_params(io.size());
            unsafe {
                let stream = &self.hal.stream;
                launch!(kernel<<<params.0, params.1, 0, stream>>>(io.as_device_ptr())).unwrap();
            }
            self.hal.stream.synchronize().unwrap();
        });
    }
}

pub type CudaCircuitHalSha256 = CudaCircuitHal<CudaHashSha256>;
pub type CudaCircuitHalPoseidon = CudaCircuitHal<CudaHashPoseidon>;
pub type CudaCircuitHalPoseidon2 = CudaCircuitHal<CudaHashPoseidon2>;

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use risc0_core::field::baby_bear::BabyBear;
    use risc0_zkp::{
        core::hash::sha::Sha256HashSuite,
        hal::{cpu::CpuHal, cuda::CudaHalSha256},
    };
    use test_log::test;

    use crate::{cpu::CpuCircuitHal, CircuitImpl};

    #[test]
    fn eval_check() {
        const PO2: usize = 4;
        let circuit = CircuitImpl::new();
        let cpu_hal: CpuHal<BabyBear> = CpuHal::new(Sha256HashSuite::new_suite());
        let cpu_eval = CpuCircuitHal::new(&circuit);
        let gpu_hal = Rc::new(CudaHalSha256::new());
        let gpu_eval = super::CudaCircuitHalSha256::new(gpu_hal.clone());
        crate::testutil::eval_check(&cpu_hal, cpu_eval, gpu_hal.as_ref(), gpu_eval, PO2);
    }
}
