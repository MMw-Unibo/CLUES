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

use crate::{broker::Broker, vdctrl::VirtDevCtrl};

pub struct Demux {
    vdctrl: VirtDevCtrl,
    broker: Broker,
}

impl Demux {
    pub fn new(broker: Broker, vdctrl: VirtDevCtrl) -> Self {
        Self { broker, vdctrl }
    }

    pub fn dispatch(&mut self, msg: msg::Msg) {
        println!("[demux] send: {msg:?}");
        self.vdctrl.instantiate_if_new(&mut self.broker, msg.addr);
        self.broker.publish(format!("{:08x}", msg.addr), msg)
    }
}
