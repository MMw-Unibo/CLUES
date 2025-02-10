// Copyright 2024 University of Bologna
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
//

// LoRa end-device for test purposes

use lora::{self, opcodes::*, *};
use std::{thread, time};

// Set spreading factor (SF7 - SF12)
const SF: SpreadingFactor = SpreadingFactor::SF7;

// Set center frequency
const FREQ: u32 = 868100000; // in Mhz! (868.1)

// Default test payload
const PAYLOAD: &str = "TEST MESSAGE";

// List of addresses to emulate device variety
const ADDR_LST: [u64; 10] = [0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9];

pub fn main() -> Result<(), lora::Error> {
    let mut lora = Lora::new(Configs {
        sync_word: 0x12, // default sync word for non-LoRaWAN, private networks
        frf: Frf { freq: FREQ },
        modem_config1: ModemConfig1 {
            bw: Bandwidth::KHz125,
            coding_rate: CodingRate::CR4_5,
            implicit_header_mode_on: false,
        },
        modem_config2: ModemConfig2 {
            sf: SF,
            tx_continuous_mode: false,
            rx_payload_crc_on: true,
            symb_timeout_msb: 0,
        },
        symb_timeout_lsb: match SF {
            SpreadingFactor::SF10 | SpreadingFactor::SF11 | SpreadingFactor::SF12 => 5,
            _ => 8,
        },
        modem_config3: ModemConfig3 {
            low_data_rate_optimize: match SF {
                SpreadingFactor::SF11 | SpreadingFactor::SF12 => true,
                _ => false,
            },
            agc_auto_on: true,
        },
        lna: Lna {
            lna_gain: LnaGain::G1,
            lna_boost_hf: true,
        },
        max_payload: 128,
    })?;

    lora.config_pa_ramp_time(PaRampTime::US50)?;

    lora.config_power(23)?;

    println!(
        "Send packets at {:#?} on {:.6} Mhz.",
        SF,
        (FREQ as f64) / 1000000.0
    );
    println!("------------------");

    loop {
        let msg = {
            use rand::seq::SliceRandom;
            String::from("{\"addr\":\"")
                + &ADDR_LST
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .to_string()
                + "\",\"payload\":\""
                + PAYLOAD
                + "\"}"
        };

        println!("send: {}", &msg);
        lora.transmit(&msg.as_bytes())?;
        thread::sleep(time::Duration::from_secs(5))
    }
}
