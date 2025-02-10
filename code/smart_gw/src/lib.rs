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

pub mod broker;
pub mod demux;
pub mod vdctrl;

use lora::{self, opcodes::*, *};
use std::fmt;
use std::{thread, time};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Lora(lora::Error),
    Msg(msg::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Lora(ref err) => write!(f, "Lora error: {err}"),
            Error::Msg(ref err) => write!(f, "Msg error: {err}"),
        }
    }
}

pub fn init_lora() -> Result<Lora> {
    // Set spreading factor (SF7 - SF12)
    const SF: SpreadingFactor = SpreadingFactor::SF7;

    // Set center frequency
    const FREQ: u32 = 868100000; // in Mhz! (868.1)

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
    })
    .map_err(Error::Lora)?;

    lora.op_mode(Mode::RxContinuous).map_err(Error::Lora)?;

    println!(
        "Listening at {:#?} on {:.6} Mhz.",
        SF,
        (FREQ as f64) / 1000000.0
    );
    println!("------------------");

    Ok(lora)
}

// Blocking reception method
pub fn recv(lora: &mut Lora) -> Result<Vec<u8>> {
    loop {
        if let Some(r) = lora.try_receive().map_err(Error::Lora)? {
            println!(
                "receive: {:#?} ({} bytes), RSS: {} dBm, SNR: {}",
                &r.data,
                r.data.len(),
                r.rss,
                r.snr
            );
            return Ok(r.data);
        }
        thread::sleep(time::Duration::from_millis(1))
    }
}
