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

use broker::{Broker, ALL};
use demux::Demux;
use smart_gw::*;
use vdctrl::VirtDevCtrl;

use std::{env, process, thread, time::Duration};

pub fn main() -> ! {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 || (args.len() == 2 && args[1] != "--lora") {
        help()
    }

    // create pub/sub broker
    let mut broker = Broker::new();

    // subscribe the virt dev ctrl to all topics
    let global_sub = broker.subscribe(ALL);
    let vdctrl = VirtDevCtrl::new(global_sub);

    // create demultiplexer
    let mut demux = Demux::new(broker, vdctrl);

    if args.len() == 2 {
        // init lora interface
        let mut lora = init_lora().unwrap();

        // Main loop
        loop {
            let msg = {
                let bytes = recv(&mut lora).unwrap();
                match msg::deserialize(bytes.as_slice()) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("{e}");
                        continue;
                    }
                }
            };
            demux.dispatch(msg);
        }
    } else {
        // Main loop
        loop {
            let msg = {
                let bytes = emu_recv().unwrap();
                match msg::deserialize(bytes.as_slice()) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("{e}");
                        continue;
                    }
                }
            };
            demux.dispatch(msg);
        }
    }
}

// Default test payload
const PAYLOAD: &str = "TEST MESSAGE";

// List of addresses to emulate device variety
const ADDR_LST: [u64; 10] = [0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9];

fn emu_recv() -> Result<Vec<u8>> {
    use rand::seq::SliceRandom;
    thread::sleep(Duration::from_secs(1));
    msg::Msg {
        addr: *ADDR_LST.choose(&mut rand::thread_rng()).unwrap(),
        fcnt: 0,
        payload: PAYLOAD.as_bytes().to_vec(),
    }
    .serialize()
    .map_err(Error::Msg)
}

fn help() -> ! {
    println!("Usage: smart_gw [--lora]");
    process::exit(1)
}
