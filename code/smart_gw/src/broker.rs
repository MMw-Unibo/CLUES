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

// Pub/sub broker of wasmer_wasix::Pipe to wasm Stdin

use std::io::Write;
use wasmer_wasix::Pipe;

use msg::Msg;

pub const ALL: String = String::new();

pub struct Broker {
    subs: Vec<(String, Pipe)>,
}

impl Broker {
    pub fn new() -> Self {
        Self { subs: Vec::new() }
    }

    pub fn subscribe(&mut self, topic: String) -> Pipe {
        println!("[broker] new subscription on topic {topic}.");
        // create sender/receiver, store send Pipe and return receiver
        let (sender, receiver) = Pipe::channel();
        self.subs.push((topic, sender));
        receiver
    }

    pub fn publish(&mut self, topic: String, msg: Msg) {
        // write on all pub Pipes that match topic
        // (this may be optimizable with some algo theory)
        for (t, s) in &mut self.subs {
            if topic.starts_with(&t[..]) {
                msg.serialize_into(s.by_ref())
                    .expect(&format!("failed to write on pipe {s:?} (subbed to: {t})")[..]);
            }
        }
    }
}
