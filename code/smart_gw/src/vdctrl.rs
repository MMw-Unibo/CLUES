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

use crate::broker::Broker;

use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use wasmer::{Module, Store};
use wasmer_wasix::{Pipe, WasiEnv};

pub struct VirtDevCtrl {
    registry: HashMap<u64, thread::JoinHandle<()>>,
    _listener: JoinHandle<()>,
}

impl VirtDevCtrl {
    pub fn new(sub_pipe: Pipe) -> Self {
        Self {
            registry: HashMap::new(),
            _listener: Self::listen(sub_pipe),
        }
    }

    fn listen(sub_pipe: Pipe) -> JoinHandle<()> {
        let mut receiver = BufReader::new(sub_pipe);
        thread::spawn(move || loop {
            let msg = msg::deserialize_from(receiver.by_ref()).expect("failed to read from pipe");
            println!("[vdctrl] recv: {:?}", msg);
            let todo = true; // TODO: Implement vdctrl logic on msg
            thread::sleep(Duration::from_millis(20));
        })
    }

    pub fn instantiate_if_new(&mut self, broker: &mut Broker, deveui: u64) {
        if !self.registry.contains_key(&deveui) {
            let handle = {
                let wasm_path = self.discover_service(deveui);
                let sub_pipe = broker.subscribe(format!("{deveui:08x}"));
                self.run_virt_dev(sub_pipe, deveui, wasm_path)
            };
            self.registry.insert(deveui, handle);
        }
    }

    fn run_virt_dev(&mut self, sub_pipe: Pipe, deveui: u64, wasm_path: String) -> JoinHandle<()> {
        thread::spawn(move || {
            let wasm_bytes = std::fs::read(&wasm_path)
                .expect(format!("Wasm binary expected at {}", wasm_path).as_str());
            let mut store = Store::default();
            let module = Module::new(&store, wasm_bytes).expect("failed to create wasm module");
            WasiEnv::builder(format!("vd-{:08x}", deveui))
                .stdin(Box::new(sub_pipe))
                .run_with_store(module, &mut store)
                .expect("failed to run virt dev wasm");
        })
    }

    /// Call to hyperledger to get binary
    fn discover_service(&self, _deveui: u64) -> String {
        let todo = true; // TODO: Implement service discovery
        "./target/wasm32-wasi/release/virt_dev.wasm".to_string()
    }
}
