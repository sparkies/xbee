#![feature(io)]
extern crate serial;

use serial::prelude::*;

use std::io::prelude::*;
use std::time::Duration;
use std::ffi::OsStr;

#[derive(Debug)]
pub enum Error {
    SerialError(serial::core::Error),
    IoError(std::io::Error),
}

pub struct Xbee {
    port: serial::SystemPort,
}

impl Xbee {
    pub fn new<T: AsRef<OsStr> + ?Sized>(port: &T) -> Result<Xbee, Error> {
        let mut port = serial::open(port).map_err(Error::SerialError)?;

        port.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud9600)?;
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        }).map_err(Error::SerialError)?;

        port.set_timeout(Duration::from_millis(10))
            .map_err(Error::SerialError)?;

        Ok(Xbee {
            port: port,
        })
    }

    pub fn write_raw<T: Into<String>>(&mut self, data: T) -> Result<usize, Error> {
        self.port
            .write(data.into().as_bytes())
            .map_err(Error::IoError)
    }

    pub fn read_raw(&mut self) -> String {
        let mut output = String::new();

        for _ in 1..300 {
            let mut data = [0; 1024];
            
            if let Ok(amount) = self.port.read(&mut data[..]) {
                output += std::str::from_utf8(&data[0..amount]).unwrap_or("");
            }

            if output.contains('\r') {
                break;
            }
        }

        output
    }
}
