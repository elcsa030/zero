// This code is automatically generated

use anyhow::Result;
use risc0_zkp::{
    adapter::{CircuitStep, CircuitStepAccum, CircuitStepContext, CircuitStepDef, CustomStep},
    core::fp::Fp,
};

use super::CircuitImpl;

const DEF: CircuitStepDef = CircuitStepDef {
    block: &[
        CircuitStep::Const(2, "external/risc0/risc0/zkvm/circuit/data_regs.cpp:28"),
        CircuitStep::Const(11, "external/risc0/risc0/zkvm/circuit/data_regs.cpp:28"),
        CircuitStep::Const(2013265910, "external/risc0/risc0/zkvm/circuit/edsl.h:122"),
        CircuitStep::Const(1, "circuits/rv32im-legacy/port.cpp:207"),
        CircuitStep::Get(
            0,
            0,
            0,
            "external/risc0/risc0/zkvm/circuit/data_regs.cpp:23",
        ),
        CircuitStep::Get(
            0,
            3,
            0,
            "external/risc0/risc0/zkvm/circuit/data_regs.cpp:25",
        ),
        CircuitStep::Get(
            0,
            4,
            0,
            "external/risc0/risc0/zkvm/circuit/data_regs.cpp:30",
        ),
        CircuitStep::Get(
            0,
            5,
            0,
            "external/risc0/risc0/zkvm/circuit/data_regs.cpp:37",
        ),
        CircuitStep::Get(
            0,
            6,
            0,
            "external/risc0/risc0/zkvm/circuit/data_regs.cpp:55",
        ),
        CircuitStep::Get(
            0,
            1,
            0,
            "external/risc0/risc0/zkvm/circuit/data_regs.cpp:63",
        ),
        CircuitStep::Get(
            0,
            2,
            0,
            "external/risc0/risc0/zkvm/circuit/data_regs.cpp:143",
        ),
        CircuitStep::Add(9, 10, "external/risc0/risc0/zkvm/circuit/data_regs.cpp:192"),
        CircuitStep::Add(11, 5, "external/risc0/risc0/zkvm/circuit/data_regs.cpp:192"),
        CircuitStep::Add(12, 6, "external/risc0/risc0/zkvm/circuit/data_regs.cpp:192"),
        CircuitStep::Add(13, 7, "external/risc0/risc0/zkvm/circuit/data_regs.cpp:192"),
        CircuitStep::Add(14, 8, "external/risc0/risc0/zkvm/circuit/data_regs.cpp:192"),
        CircuitStep::If(
            15,
            &[
                CircuitStep::Get(
                    2,
                    148,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:37",
                ),
                CircuitStep::Get(
                    2,
                    128,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:40",
                ),
                CircuitStep::Get(
                    2,
                    144,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:41",
                ),
                CircuitStep::Get(
                    2,
                    129,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:40",
                ),
                CircuitStep::Get(
                    2,
                    145,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:41",
                ),
                CircuitStep::Get(
                    2,
                    130,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:40",
                ),
                CircuitStep::Get(
                    2,
                    146,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:41",
                ),
                CircuitStep::Get(
                    2,
                    131,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:40",
                ),
                CircuitStep::Get(
                    2,
                    147,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:41",
                ),
                CircuitStep::GetGlobal(3, 0, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49"),
                CircuitStep::Mul(4, 25, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49"),
                CircuitStep::Add(26, 3, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49"),
                CircuitStep::Mul(
                    17,
                    25,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    27,
                    28,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    26,
                    28,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Mul(
                    19,
                    25,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    29,
                    31,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    30,
                    31,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Mul(
                    21,
                    25,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    32,
                    34,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    33,
                    34,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Mul(
                    23,
                    25,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    35,
                    37,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    36,
                    37,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Mul(
                    16,
                    25,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(40, 3, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49"),
                CircuitStep::Mul(
                    18,
                    25,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    41,
                    42,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    40,
                    42,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Mul(
                    20,
                    25,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    43,
                    45,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    44,
                    45,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Mul(
                    22,
                    25,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    46,
                    48,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    47,
                    48,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Mul(
                    24,
                    25,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    49,
                    51,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Add(
                    50,
                    51,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:49",
                ),
                CircuitStep::Sub(3, 5, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61"),
                CircuitStep::Get(
                    4,
                    0,
                    1,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Mul(
                    54,
                    55,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Add(5, 56, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61"),
                CircuitStep::Get(
                    4,
                    1,
                    1,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Mul(
                    54,
                    58,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Add(5, 59, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61"),
                CircuitStep::Get(
                    4,
                    2,
                    1,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Mul(
                    54,
                    61,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Add(5, 62, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61"),
                CircuitStep::Get(
                    4,
                    3,
                    1,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Mul(
                    54,
                    64,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Add(5, 65, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61"),
                CircuitStep::Get(
                    4,
                    4,
                    1,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Mul(
                    54,
                    67,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Add(5, 68, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61"),
                CircuitStep::Get(
                    4,
                    5,
                    1,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Mul(
                    54,
                    70,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Add(5, 71, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61"),
                CircuitStep::Get(
                    4,
                    6,
                    1,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Mul(
                    54,
                    73,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Add(5, 74, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61"),
                CircuitStep::Get(
                    4,
                    7,
                    1,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Mul(
                    54,
                    76,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61",
                ),
                CircuitStep::Add(5, 77, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:61"),
                CircuitStep::Mul(
                    57,
                    38,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Mul(
                    60,
                    39,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Mul(
                    63,
                    39,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Add(
                    80,
                    81,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Mul(
                    66,
                    39,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Add(
                    82,
                    83,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Mul(84, 2, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70"),
                CircuitStep::Add(
                    79,
                    85,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Set(
                    4,
                    86,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Mul(
                    57,
                    39,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:71",
                ),
                CircuitStep::Mul(
                    60,
                    38,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:71",
                ),
                CircuitStep::Add(
                    87,
                    88,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:71",
                ),
                CircuitStep::Add(
                    81,
                    83,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:71",
                ),
                CircuitStep::Mul(90, 2, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:71"),
                CircuitStep::Add(
                    89,
                    91,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:71",
                ),
                CircuitStep::Set(
                    4,
                    92,
                    1,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:71",
                ),
                CircuitStep::Add(
                    87,
                    80,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:72",
                ),
                CircuitStep::Mul(
                    63,
                    38,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:72",
                ),
                CircuitStep::Add(
                    93,
                    94,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:72",
                ),
                CircuitStep::Mul(83, 2, "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:72"),
                CircuitStep::Add(
                    95,
                    96,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:72",
                ),
                CircuitStep::Set(
                    4,
                    97,
                    2,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:72",
                ),
                CircuitStep::Add(
                    93,
                    81,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:73",
                ),
                CircuitStep::Mul(
                    66,
                    38,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:73",
                ),
                CircuitStep::Add(
                    98,
                    99,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:73",
                ),
                CircuitStep::Set(
                    4,
                    100,
                    3,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:73",
                ),
                CircuitStep::Mul(
                    69,
                    52,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Mul(
                    72,
                    53,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Mul(
                    75,
                    53,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Add(
                    102,
                    103,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Mul(
                    78,
                    53,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Add(
                    104,
                    105,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Mul(
                    106,
                    2,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Add(
                    101,
                    107,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Set(
                    4,
                    108,
                    4,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:70",
                ),
                CircuitStep::Mul(
                    69,
                    53,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:71",
                ),
                CircuitStep::Mul(
                    72,
                    52,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:71",
                ),
                CircuitStep::Add(
                    109,
                    110,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:71",
                ),
                CircuitStep::Add(
                    103,
                    105,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:71",
                ),
                CircuitStep::Mul(
                    112,
                    2,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:71",
                ),
                CircuitStep::Add(
                    111,
                    113,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:71",
                ),
                CircuitStep::Set(
                    4,
                    114,
                    5,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:71",
                ),
                CircuitStep::Add(
                    109,
                    102,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:72",
                ),
                CircuitStep::Mul(
                    75,
                    52,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:72",
                ),
                CircuitStep::Add(
                    115,
                    116,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:72",
                ),
                CircuitStep::Mul(
                    105,
                    2,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:72",
                ),
                CircuitStep::Add(
                    117,
                    118,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:72",
                ),
                CircuitStep::Set(
                    4,
                    119,
                    6,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:72",
                ),
                CircuitStep::Add(
                    115,
                    103,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:73",
                ),
                CircuitStep::Mul(
                    78,
                    52,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:73",
                ),
                CircuitStep::Add(
                    120,
                    121,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:73",
                ),
                CircuitStep::Set(
                    4,
                    122,
                    7,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:73",
                ),
                CircuitStep::Get(
                    4,
                    0,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:80",
                ),
                CircuitStep::Get(
                    4,
                    1,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:80",
                ),
                CircuitStep::Get(
                    4,
                    2,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:80",
                ),
                CircuitStep::Get(
                    4,
                    3,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:80",
                ),
                CircuitStep::Mul(
                    123,
                    123,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:81",
                ),
                CircuitStep::Mul(
                    124,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:81",
                ),
                CircuitStep::Mul(
                    128,
                    126,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:81",
                ),
                CircuitStep::Mul(
                    125,
                    125,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:81",
                ),
                CircuitStep::Sub(
                    129,
                    130,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:81",
                ),
                CircuitStep::Mul(
                    131,
                    1,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:81",
                ),
                CircuitStep::Add(
                    127,
                    132,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:81",
                ),
                CircuitStep::Mul(
                    123,
                    0,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:82",
                ),
                CircuitStep::Mul(
                    134,
                    125,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:82",
                ),
                CircuitStep::Mul(
                    124,
                    124,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:82",
                ),
                CircuitStep::Sub(
                    135,
                    136,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:82",
                ),
                CircuitStep::Mul(
                    126,
                    126,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:82",
                ),
                CircuitStep::Mul(
                    138,
                    1,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:82",
                ),
                CircuitStep::Add(
                    137,
                    139,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:82",
                ),
                CircuitStep::Mul(
                    133,
                    133,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:83",
                ),
                CircuitStep::Mul(
                    140,
                    1,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:83",
                ),
                CircuitStep::Mul(
                    142,
                    140,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:83",
                ),
                CircuitStep::Add(
                    141,
                    143,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:83",
                ),
                CircuitStep::Set(
                    4,
                    144,
                    8,
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:84",
                ),
                CircuitStep::If(
                    8,
                    &[
                        CircuitStep::Nondet(
                            &[
                                CircuitStep::Get(
                                    4,
                                    8,
                                    0,
                                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:89",
                                ),
                                CircuitStep::Inv(
                                    145,
                                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:89",
                                ),
                                CircuitStep::Set(
                                    4,
                                    146,
                                    9,
                                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:89",
                                ),
                            ],
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:89",
                        ),
                        CircuitStep::Get(
                            4,
                            4,
                            0,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:91",
                        ),
                        CircuitStep::Sub(
                            123,
                            145,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:91",
                        ),
                        CircuitStep::EqZero(
                            146,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:91",
                        ),
                        CircuitStep::Get(
                            4,
                            5,
                            0,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:91",
                        ),
                        CircuitStep::Sub(
                            124,
                            147,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:91",
                        ),
                        CircuitStep::EqZero(
                            148,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:91",
                        ),
                        CircuitStep::Get(
                            4,
                            6,
                            0,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:91",
                        ),
                        CircuitStep::Sub(
                            125,
                            149,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:91",
                        ),
                        CircuitStep::EqZero(
                            150,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:91",
                        ),
                        CircuitStep::Get(
                            4,
                            7,
                            0,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:91",
                        ),
                        CircuitStep::Sub(
                            126,
                            151,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:91",
                        ),
                        CircuitStep::EqZero(
                            152,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:91",
                        ),
                        CircuitStep::Get(
                            4,
                            9,
                            0,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:93",
                        ),
                        CircuitStep::Get(
                            4,
                            8,
                            0,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:93",
                        ),
                        CircuitStep::Mul(
                            153,
                            154,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:93",
                        ),
                        CircuitStep::Sub(
                            155,
                            3,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:93",
                        ),
                        CircuitStep::EqZero(
                            156,
                            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:93",
                        ),
                    ],
                    "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:88",
                ),
            ],
            "external/risc0/risc0/zkvm/circuit/accum_regs.cpp:28",
        ),
    ],
    ret: 3,
};

impl<S: CustomStep> CircuitStepAccum<S> for CircuitImpl {
    #[allow(unused)]
    fn step_accum(
        &self,
        ctx: &CircuitStepContext,
        custom: &mut S,
        args: &mut [&mut [Fp]],
    ) -> Result<Fp> {
        DEF.step(ctx, custom, args)
    }
}
