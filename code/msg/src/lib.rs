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

use serde::{Deserialize, Serialize};
use std::fmt;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Fmt(bincode::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Fmt(ref err) => write!(f, "Bad msg fmt: {err}"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Msg {
    pub addr: u64,
    pub fcnt: u32,
    pub payload: Vec<u8>,
}

impl Msg {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(&self).map_err(Error::Fmt)
    }

    pub fn serialize_into<W>(&self, writer: W) -> Result<()>
    where
        W: std::io::Write,
    {
        bincode::serialize_into(writer, &self).map_err(Error::Fmt)
    }
}

pub fn deserialize(bytes: &[u8]) -> Result<Msg> {
    bincode::deserialize(bytes).map_err(Error::Fmt)
}

pub fn deserialize_from<R>(reader: R) -> Result<Msg>
where
    R: std::io::Read,
{
    bincode::deserialize_from(reader).map_err(Error::Fmt)
}
