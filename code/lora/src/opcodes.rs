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

use std::fmt;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ModeNotSupported(u8),
    FrequencyOutOfRange(u32),
    SymbTimeoutMsbOverflow(u8),
    OutputPowerOverflow(u8),
    MaxPowerOverflow(u8),
    PaRampTimeNotSupported(u8),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::ModeNotSupported(v) => write!(f, "OpMode {v:02X?} not supported"),
            Error::FrequencyOutOfRange(freq) => write!(f, "Frequency out of range ({freq} Hz)"),
            Error::PaRampTimeNotSupported(v) => write!(f, "Unknown PaRamp time value: {v:02X?}"),
            Error::SymbTimeoutMsbOverflow(v) => {
                write!(f, "SymbTimeoutMsb overflow (max: 0x03): {v:02X?})")
            }
            Error::OutputPowerOverflow(v) => {
                write!(f, "OutputPower overflow (max: 0x0F): {v:02X?}")
            }
            Error::MaxPowerOverflow(v) => {
                write!(f, "MaxPower overflow (max: 0x07): {v:02X?}")
            }
        }
    }
}

// #############################################
// LoRa SX1276 Registers
// #############################################

// see [SX1276/7/8/9 Datasheet Rev.7, Tab. 41]
#[derive(Debug)]
pub enum Reg {
    Fifo = 0x00, //
    // Common Register Settings
    OpMode,        // Operating mode
    FrfMsb = 0x06, // Carrier frequency
    FrfMid,        //
    FrfLsb,        //
    // Registers for RF blocks
    PaConfig, // Power Amplification
    PaRamp,   //
    Ocp,      // Over Current Protection
    Lna,      // Low Noise Amplifier
    // LoRa page registers
    FifoAddrPtr,            // FIFO SPI pointer
    FifoTxBaseAddr,         // Start Tx data
    FifoRxBaseAddr,         // Start Rx data
    FifoRxCurrentAddr,      // Start address of last packet received
    IrqFlagsMask,           // Optional IRQ flag mask
    IrqFlags,               //
    RxNbBytes,              //
    RxHeaderCntValueMsb,    //
    RxHeaderCntValueLsb,    //
    RxPacketCntValueMsb,    //
    RxPacketCntValueLsb,    //
    ModemStat,              //
    PktSnrValue,            //
    PktRssiValue,           //
    RssiValue,              //
    HopChannel,             //
    ModemConfig1,           //
    ModemConfig2,           //
    SymbTimeoutLsb,         //
    PreambleMsb,            //
    PreambleLsb,            //
    PayloadLength,          //
    MaxPayloadLength,       //
    HopPeriod,              //
    FifoRxByteAddr,         // Address of last byte written in FIFO
    ModemConfig3,           //
    FeiMsb = 0x28,          //
    FeiMid,                 //
    FeiLsb,                 //
    RssiWideband = 0x2C,    //
    IfFreq1 = 0x2F,         //
    IfFreq2,                //
    DetectOptimize,         //
    InvertIQ = 0x33,        //
    HighBwOptimize1 = 0x36, //
    DetectionThreshold,     //
    SyncWord = 0x39,        //
    HighBwOptimize2,        //
    InvertIQ2,              //
    DioMapping1 = 0x40,     //
    DioMapping2,            //
    Version,                //
    Tcxo = 0x4B,            //
    PaDac,                  //
    FormerTemp = 0x5B,      // Stored temperature during the former IQ Calibration
    AgcRef = 0x61,          //
    AgcThresh1,             //
    AgcThresh2,             //
    AgcThresh3,             //
    Pll = 0x70,             //
}

// Configurations
pub struct Configs {
    pub sync_word: u8,
    pub frf: Frf,
    pub lna: Lna,
    pub modem_config1: ModemConfig1,
    pub modem_config2: ModemConfig2,
    pub symb_timeout_lsb: u8,
    pub max_payload: u8,
    pub modem_config3: ModemConfig3,
}

// RegOpMode

#[derive(Debug)]
pub enum Mode {
    Sleep = 0x00,
    Stdby,
    FsTx,
    Tx,
    FsRx,
    RxContinuous,
    RxSingle,
    Cad,
}

impl TryFrom<u8> for Mode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value & 0x07 {
            0x00 => Ok(Mode::Sleep),
            0x01 => Ok(Mode::Stdby),
            0x02 => Ok(Mode::FsTx),
            0x03 => Ok(Mode::Tx),
            0x04 => Ok(Mode::FsRx),
            0x05 => Ok(Mode::RxContinuous),
            0x06 => Ok(Mode::RxSingle),
            0x07 => Ok(Mode::Cad),
            v => Err(Error::ModeNotSupported(v)),
        }
    }
}

#[derive(Debug)]
pub struct OpMode {
    pub long_range_mode: bool,
    pub access_shared_reg: bool,
    pub low_frequency_mode_on: bool,
    pub mode: Mode,
}

impl OpMode {
    pub fn serialize(self) -> u8 {
        (self.long_range_mode as u8) << 7
            | (self.access_shared_reg as u8) << 6
            | (self.long_range_mode as u8) << 3
            | self.mode as u8
    }

    pub fn deserialize(op_mode: u8) -> OpMode {
        OpMode {
            long_range_mode: op_mode & 0x08 != 0,
            access_shared_reg: op_mode & 0x40 != 0,
            low_frequency_mode_on: op_mode & 0x08 != 0,
            mode: Mode::try_from(op_mode & 0x07).unwrap(),
        }
    }
}

// RegFrf

#[derive(Debug)]
pub struct Frf {
    pub freq: u32,
}

impl Frf {
    pub fn serialize(self) -> Result<(u8, u8, u8)> {
        // set frequency (see [SX1276/7/8/9 Datasheet Rev.7, Sec. 3.3.3])
        // e.g. 868.1 MHz * 2^19 / 32 MHz = 1101 1001 0000 0110 0110 0110 = D9 06 66s
        let frf = ((self.freq as u64) << 19) / 32000000;
        if frf > (1u64 << 24) - 1 {
            return Err(Error::FrequencyOutOfRange(self.freq));
        }
        Ok(((frf >> 16) as u8, (frf >> 8) as u8, (frf >> 0) as u8))
    }
}

// RegPaConfig

#[derive(Debug)]
pub struct PaConfig {
    pub pa_select_boost: bool,
    pub max_power: u8,
    pub output_power: u8,
}

impl PaConfig {
    pub fn serialize(self) -> Result<u8> {
        if self.output_power > 0xF {
            return Err(Error::OutputPowerOverflow(self.output_power));
        }
        if self.max_power > 0x7 {
            return Err(Error::MaxPowerOverflow(self.max_power));
        }
        Ok((self.pa_select_boost as u8) << 7
            | (self.max_power as u8) << 4
            | (self.output_power & 0xF))
    }
}

// RegPaRamp

#[derive(Debug)]
pub enum PaRampTime {
    MS3_4 = 0x00,
    MS2,
    MS1,
    US500,
    US250,
    US125,
    US100,
    US62,
    US50,
    US40,
    US31,
    US25,
    US20,
    US15,
    US12,
    US10,
}

impl TryFrom<u8> for PaRampTime {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value & 0x07 {
            0x00 => Ok(PaRampTime::MS3_4),
            0x01 => Ok(PaRampTime::MS2),
            0x02 => Ok(PaRampTime::MS1),
            0x03 => Ok(PaRampTime::US500),
            0x04 => Ok(PaRampTime::US250),
            0x05 => Ok(PaRampTime::US125),
            0x06 => Ok(PaRampTime::US100),
            0x07 => Ok(PaRampTime::US62),
            0x08 => Ok(PaRampTime::US50),
            0x09 => Ok(PaRampTime::US40),
            0x0A => Ok(PaRampTime::US31),
            0x0B => Ok(PaRampTime::US25),
            0x0C => Ok(PaRampTime::US20),
            0x0D => Ok(PaRampTime::US15),
            0x0E => Ok(PaRampTime::US12),
            0x0F => Ok(PaRampTime::US10),
            v => Err(Error::PaRampTimeNotSupported(v)),
        }
    }
}

#[derive(Debug)]
pub struct PaRamp {
    pub time: PaRampTime,
    msb: u8,
}

impl PaRamp {
    pub fn deserialize(pa_ramp: u8) -> PaRamp {
        PaRamp {
            time: PaRampTime::try_from(pa_ramp & 0x0F).unwrap(),
            msb: pa_ramp & 0xF0,
        }
    }

    pub fn serialize(self) -> u8 {
        self.time as u8 | self.msb
    }
}

// RegLna

#[derive(Debug)]
pub enum LnaGain {
    G1 = 0x01,
    G2,
    G3,
    G4,
    G5,
    G6,
}

#[derive(Debug)]
pub struct Lna {
    pub lna_gain: LnaGain,
    pub lna_boost_hf: bool,
}

impl Lna {
    pub fn serialize(self) -> u8 {
        (self.lna_gain as u8) << 5 | (self.lna_boost_hf as u8) << 1 | self.lna_boost_hf as u8
    }
}

// RegIrqFlags
pub enum IrqFlag {
    RxTimeout = 0x80,         // Valid Lora signal detected during CAD operation
    RxDone = 0x40,            // FHSS change channel interrupt
    PayloadCrcError = 0x20,   // CAD complete
    ValidHeader = 0x10,       // FIFO Payload transmission complete interrupt
    TxDone = 0x08,            // Valid header received in Rx
    CadDone = 0x04,           // Payload CRC error interrupt
    FhssChangeChannel = 0x02, // Packet reception complete interrupt
    CadDetected = 0x01,       // Timeout interrupt
}

// RegModemConfig1

#[derive(Debug)]
pub enum Bandwidth {
    KHz7_8 = 0x00,
    KHz10_4,
    KHz15_6,
    KHz20_8,
    KHz31_25,
    KHz41_7,
    KHz62_5,
    KHz125,
    KHz250,
    KHz500,
}

#[derive(Debug)]
pub enum CodingRate {
    CR4_5 = 0x01,
    CR4_6,
    CR4_7,
    CR4_8,
}

#[derive(Debug)]
pub struct ModemConfig1 {
    pub bw: Bandwidth,
    pub coding_rate: CodingRate,
    pub implicit_header_mode_on: bool,
}

impl ModemConfig1 {
    pub fn serialize(self) -> u8 {
        (self.bw as u8) << 4 | (self.coding_rate as u8) << 1 | self.implicit_header_mode_on as u8
    }
}

// RegModemConfig2

#[derive(Debug, Clone, Copy)]
pub enum SpreadingFactor {
    SF6 = 0x06,
    SF7,
    SF8,
    SF9,
    SF10,
    SF11,
    SF12,
}

#[derive(Debug)]
pub struct ModemConfig2 {
    pub sf: SpreadingFactor,
    pub tx_continuous_mode: bool,
    pub rx_payload_crc_on: bool,
    pub symb_timeout_msb: u8,
}

impl ModemConfig2 {
    pub fn serialize(self) -> Result<u8> {
        if self.symb_timeout_msb > 0x3 {
            return Err(Error::SymbTimeoutMsbOverflow(self.symb_timeout_msb));
        }
        Ok((self.sf as u8) << 4
            | (self.tx_continuous_mode as u8) << 3
            | (self.rx_payload_crc_on as u8) << 2
            | self.symb_timeout_msb)
    }
}

// RegModemConfig3

#[derive(Debug)]
pub struct ModemConfig3 {
    pub low_data_rate_optimize: bool,
    pub agc_auto_on: bool,
}

impl ModemConfig3 {
    pub fn serialize(self) -> u8 {
        (self.low_data_rate_optimize as u8) << 3 | (self.agc_auto_on as u8) << 2
    }
}

// DioMapping1

#[derive(Debug)]
pub enum Dio0 {
    RxDone = 0x00,
    TxDone,
    CadDone,
    Nop = 0x11,
}

#[derive(Debug)]
pub enum Dio1 {
    RxTimeout = 0x00,
    FhssChangeChannel,
    CadDetected,
    Nop = 0x11,
}

#[derive(Debug)]
pub enum Dio2 {
    FhssChangeChannel = 0x00,
    Nop = 0x11,
}

#[derive(Debug)]
pub enum Dio3 {
    CadDone = 0x00,
    ValidHeader,
    PayloadCrcError,
    Nop = 0x11,
}

#[derive(Debug)]
pub struct DioMapping1 {
    pub dio0_mapping: Dio0,
    pub dio1_mapping: Dio1,
    pub dio2_mapping: Dio2,
    pub dio3_mapping: Dio3,
}

impl DioMapping1 {
    pub fn serialize(self) -> u8 {
        (self.dio0_mapping as u8) << 6
            | (self.dio1_mapping as u8) << 4
            | (self.dio2_mapping as u8) << 2
            | self.dio3_mapping as u8
    }
}

// DioMapping2

#[derive(Debug)]
pub enum Dio4 {
    CadDetected = 0x00,
    PllLock,
    Nop = 0x11,
}

#[derive(Debug)]
pub enum Dio5 {
    ModeReady = 0x00,
    ClkOut,
    Nop = 0x11,
}

#[derive(Debug)]
pub struct DioMapping2 {
    pub dio4_mapping: Dio4,
    pub dio5_mapping: Dio5,
}

impl DioMapping2 {
    pub fn serialize(self) -> u8 {
        (self.dio4_mapping as u8) << 6 | (self.dio5_mapping as u8) << 4
    }
}
