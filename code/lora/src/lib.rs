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

// Simple HAL for the Raspberry Pi's Dragino LoRa GPS HAT module
//
// Sources:
//  - [https://github.com/dragino/rpi-lora-tranceiver.git]
//  - [SX1276/7/8/9 Datasheet Rev.7]
//

pub mod opcodes;

use opcodes::*;
use rppal::{gpio, spi};
use std::fmt;
use std::{thread::sleep, time::Duration};

type Result<T> = std::result::Result<T, Error>;

/////////////////////////////
//                         //
// Configure these values! //
//                         //
/////////////////////////////

const SX1276_VERSION: u8 = 0x12;

// SX1276 - Raspberry Pi connections (BCM GPIO pin numbers)
const NSS: u8 = 25;
const DIO0: u8 = 4;
const RST: u8 = 17;

// #############################################
// #############################################
//

#[derive(Debug)]
pub enum Error {
    Spi(spi::Error),
    Gpio(gpio::Error),
    OpCode(opcodes::Error),
    UnknownTransceiver,
    PayloadCrcError,
    PayloadLenOver255,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Spi(ref err) => write!(f, "SPI error: {err}"),
            Error::Gpio(ref err) => write!(f, "GPIO error: {err}"),
            Error::OpCode(ref err) => write!(f, "OpCode error: {err}"),
            Error::UnknownTransceiver => write!(f, "Unrecognized transceiver."),
            Error::PayloadCrcError => write!(f, "CRC error during Rx"),
            Error::PayloadLenOver255 => write!(f, "Tx payload length exceeds 255 bytes."),
        }
    }
}

pub struct Reception {
    pub data: Vec<u8>,
    pub rss: i32,
    pub snr: i32,
}

pub struct Lora {
    nss: gpio::OutputPin,
    dio0: gpio::InputPin,
    rst: gpio::OutputPin,
    spi: spi::Spi,
}

impl Lora {
    pub fn new(configs: Configs) -> Result<Lora> {
        // PowerOn-Reset SPI access prevention
        sleep(Duration::from_millis(10));

        // Get the necessary GPIO pins handles
        let gpio = gpio::Gpio::new().map_err(Error::Gpio)?;
        let nss = gpio.get(NSS).map_err(Error::Gpio)?.into_output();
        let dio0 = gpio.get(DIO0).map_err(Error::Gpio)?.into_input();
        let rst = gpio.get(RST).map_err(Error::Gpio)?.into_output();

        // Get the SPI interface handle
        let spi = spi::Spi::new(
            spi::Bus::Spi0,
            spi::SlaveSelect::Ss0,
            500000,
            spi::Mode::Mode0,
        )
        .map_err(Error::Spi)?;

        Lora {
            nss,
            rst,
            dio0,
            spi,
        }
        .init(configs)
    }

    pub fn transmit(&mut self, payload: &[u8]) -> Result<usize> {
        if payload.len() > 255 {
            return Err(Error::PayloadLenOver255);
        }

        self.single_write(
            Reg::DioMapping1,
            DioMapping1 {
                dio0_mapping: Dio0::TxDone,
                dio1_mapping: Dio1::Nop,
                dio2_mapping: Dio2::Nop,
                dio3_mapping: Dio3::CadDone,
            }
            .serialize(),
        )?;

        // clear all radio IRQ flags
        self.single_write(Reg::IrqFlags, 0xFF)?;
        // mask all IRQs but TxDone
        self.single_write(Reg::IrqFlagsMask, !(IrqFlag::TxDone as u8))?;

        // initialize the payload size and address pointers
        self.single_write(Reg::FifoTxBaseAddr, 0x00)?;
        self.single_write(Reg::FifoAddrPtr, 0x00)?;
        self.single_write(Reg::PayloadLength, payload.len() as u8)?;

        // download buffer to the radio FIFO
        let len = self.fifo_write(payload)?;
        // now we actually start the transmission
        self.op_mode(Mode::Tx)?;

        Ok(len)
    }

    pub fn try_receive(&mut self) -> Result<Option<Reception>> {
        if self.dio0.is_low() {
            return Ok(None);
        }

        let data = self.receive_bytes()?;

        let snr = {
            let reg_value = self.single_read(Reg::PktSnrValue)?;
            if reg_value & 0x80 != 0 {
                // The SNR sign bit is 1
                // Invert and divide by 4
                -((!reg_value + 1 >> 2) as i32)
            } else {
                // Divide by 4
                (reg_value >> 2) as i32
            }
        };

        let rss = {
            const CORRECTION: i32 = -157;
            let rssi = CORRECTION + self.single_read(Reg::PktRssiValue)? as i32;
            if snr < 0 {
                rssi + snr / 4
            } else {
                rssi
            }
        };

        Ok(Some(Reception { data, snr, rss }))
    }

    pub fn op_mode_lora(&mut self) -> Result<()> {
        // This also forces sleep mode ?
        // TBD: sx1276 high freq
        self.single_write(
            Reg::OpMode,
            OpMode {
                long_range_mode: true,
                access_shared_reg: false,
                low_frequency_mode_on: true,
                mode: Mode::Sleep,
            }
            .serialize(),
        )
    }

    pub fn op_mode(&mut self, mode: Mode) -> Result<()> {
        // Overwrite mode and keep other fields as is
        let mut op_mode = OpMode::deserialize(self.single_read(Reg::OpMode)?);
        op_mode.mode = mode;
        self.single_write(Reg::OpMode, op_mode.serialize())
    }

    pub fn config_pa_ramp_time(&mut self, time: PaRampTime) -> Result<()> {
        // PA: power amplification, PA ramp: rise/fall time of PA ramp up/down
        let mut pa_ramp = PaRamp::deserialize(self.single_read(Reg::PaRamp)?);
        pa_ramp.time = time;
        self.single_write(Reg::PaRamp, pa_ramp.serialize())
    }

    // Set PA config (2-17 dBm using PA_BOOST)
    // Docs seem to indicate that 17 pw == lowest (2 dBm), 2 pw == highest (17 dBm)
    pub fn config_power(&mut self, pw: u8) -> Result<()> {
        self.single_write(
            Reg::PaConfig,
            PaConfig {
                pa_select_boost: true,
                max_power: 0x00,
                output_power: (if pw < 2 {
                    0
                } else if pw > 17 {
                    15
                } else {
                    pw - 2
                }) as u8,
            }
            .serialize()
            .map_err(Error::OpCode)?,
        )
    }

    fn init(mut self, configs: Configs) -> Result<Lora> {
        // Manual reset of the chip
        self.rst.set_low();
        sleep(Duration::from_micros(101));
        self.rst.set_high();
        sleep(Duration::from_millis(5));

        // Check SoC
        if self.single_read(Reg::Version)? == SX1276_VERSION {
            println!("init: SX1276 detected, starting.");
        } else {
            return Err(Error::UnknownTransceiver);
        }

        self.op_mode(Mode::Sleep)?;

        let c = configs;

        self.single_write(Reg::SyncWord, c.sync_word)?;

        let frf = c.frf.serialize().map_err(Error::OpCode)?;
        self.single_write(Reg::FrfMsb, frf.0)?;
        self.single_write(Reg::FrfMid, frf.1)?;
        self.single_write(Reg::FrfLsb, frf.2)?;

        self.single_write(Reg::ModemConfig1, c.modem_config1.serialize())?;
        self.single_write(
            Reg::ModemConfig2,
            c.modem_config2.serialize().map_err(Error::OpCode)?,
        )?;
        self.single_write(Reg::ModemConfig3, c.modem_config3.serialize())?;

        // Reception window length (preamble detection) in num of symb
        // (only useful in single reception op mode?)
        self.single_write(Reg::SymbTimeoutLsb, c.symb_timeout_lsb)?;

        // Enables dropping bad packets (e.g. if too long)
        self.single_write(Reg::MaxPayloadLength, c.max_payload)?;
        // period in symb between freq. hops (disable)
        self.single_write(Reg::HopPeriod, 0x00)?;

        // Set the initial SPI FIFO addr to the FIFO memory base addr
        self.copy_reg(Reg::FifoRxBaseAddr, Reg::FifoAddrPtr)?;

        // LnaGain future value may be controlled by AgcAuto
        self.single_write(Reg::Lna, c.lna.serialize())?;

        self.op_mode_lora()?;
        self.op_mode(Mode::Stdby)?; // enter standby mode (required for FIFO loading))

        Ok(self)
    }

    fn receive_bytes(&mut self) -> Result<Vec<u8>> {
        // clear rxDone
        self.single_write(Reg::IrqFlags, IrqFlag::RxDone as u8)?;

        let irq_flags = self.single_read(Reg::IrqFlags)?;

        // Check CRC
        if irq_flags & IrqFlag::PayloadCrcError as u8 != 0 {
            self.single_write(Reg::IrqFlags, IrqFlag::PayloadCrcError as u8)?;
            return Err(Error::PayloadCrcError);
        }

        let fifo_rx_current_addr = self.single_read(Reg::FifoRxCurrentAddr)?;
        self.single_write(Reg::FifoAddrPtr, fifo_rx_current_addr)?;

        let rx_nb_bytes = self.single_read(Reg::RxNbBytes)?;
        self.fifo_read(rx_nb_bytes)
    }

    fn copy_reg(&mut self, source: Reg, dest: Reg) -> Result<()> {
        let value = self.single_read(source)?;
        self.single_write(dest, value)
    }

    // FIFO read, see [SX1276/7/8/9 Datasheet Rev.7, Sec. 4.3]
    fn fifo_read(&mut self, len: u8) -> Result<Vec<u8>> {
        self.burst_read(Reg::Fifo, len)
    }

    // FIFO read, see [SX1276/7/8/9 Datasheet Rev.7, Sec. 4.3]
    fn fifo_write(&mut self, values: &[u8]) -> Result<usize> {
        self.burst_write(Reg::Fifo, values)
    }

    // SINGLE read, see [SX1276/7/8/9 Datasheet Rev.7, Sec. 4.3]
    fn single_read(&mut self, addr: Reg) -> Result<u8> {
        Ok(self.burst_read(addr, 1)?[0])
    }

    // SINGLE write, see [SX1276/7/8/9 Datasheet Rev.7, Sec. 4.3]
    fn single_write(&mut self, addr: Reg, value: u8) -> Result<()> {
        self.burst_write(addr, &[value])?;
        Ok(())
    }

    // BURST read, see [SX1276/7/8/9 Datasheet Rev.7, Sec. 4.3]
    fn burst_read(&mut self, addr: Reg, len: u8) -> Result<Vec<u8>> {
        let mut read_buffer = vec![0u8; 1 + len as usize]; // Instantiate empty buffer
        let mut write_buffer = vec![0x7F & addr as u8]; // read: first bit 0
        write_buffer.extend(vec![0u8; read_buffer.len()].iter()); // match len
        self.transfer(&mut read_buffer, &write_buffer)?;
        Ok(read_buffer[1..].to_vec())
    }

    // BURST write, see [SX1276/7/8/9 Datasheet Rev.7, Sec. 4.3]
    fn burst_write(&mut self, addr: Reg, values: &[u8]) -> Result<usize> {
        let mut write_buffer = vec![0x80 | addr as u8]; // write: first bit 1
        write_buffer.extend(values.iter()); // append values to write
        let mut read_buffer = vec![0u8; write_buffer.len()]; // match len
        self.transfer(&mut read_buffer, &write_buffer)
    }

    // This is a generic full-duplex BURST access to the SPI interface
    // as defined in [SX1276/7/8/9 Datasheet Rev.7, Sec. 4.3]
    fn transfer(&mut self, read_buffer: &mut [u8], write_buffer: &[u8]) -> Result<usize> {
        self.nss.set_low();
        let result = self
            .spi
            .transfer(read_buffer, write_buffer)
            .map_err(Error::Spi);
        self.nss.set_high();
        result
    }
}

pub fn init_sender() {}

pub fn init_receiver() {}
