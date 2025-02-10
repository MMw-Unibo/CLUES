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

use std::io::{self, Read};
use std::{thread, time::Duration};

fn main() {
    let mut stdin = io::stdin();
    loop {
        let msg = match msg::deserialize_from(stdin.by_ref()) {
            Ok(m) => m,
            Err(e) => {
                eprint!("{e}");
                continue;
            }
        };
        println!("[vd-{:08x}] recv: {msg:?}", msg.addr);
        let todo = true; // TODO: Internal driver implementation
        thread::sleep(Duration::from_millis(20));
    }
}
