#![feature(io)]
extern crate serial;

use serial::prelude::*;

use std::io::prelude::*;
use std::time::{Duration, Instant};
use std::ffi::OsStr;

#[derive(Debug)]
pub enum Error {
    SerialError(serial::core::Error),
    IoError(std::io::Error),
    ParseIntError(std::num::ParseIntError),
}

pub struct Xbee {
    port: serial::SystemPort,
    last_time: Instant,
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
            last_time: Instant::now(),
        })
    }

    pub fn write_raw<T: Into<String>>(&mut self, data: T) -> Result<usize, Error> {
        if self.last_time.elapsed().as_secs() > 8 {
            self.connect();
        }

        let result = self.port
            .write(data.into().as_bytes())
            .map_err(Error::IoError);

        self.last_time = Instant::now();

        result
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

    pub fn connect(&mut self) -> Result<bool, Error> {
        self.write_raw("+++")?;
        let resp = self.read_raw();

        Ok(resp == "OK\r")
    }

    pub fn id(&mut self) -> Result<u16, Error> {
        self.write_raw("ATID\r")?;

        let resp = self.read_raw();

        u16::from_str_radix(&resp, 16)
            .map_err(Error::ParseIntError)
    }

    pub fn set_id(&mut self, id: u16) -> Result<u16, Error> {
        self.write_raw(format!("ATID{:x}\r", id))?;

        let resp = self.read_raw();

        u16::from_str_radix(&resp, 16)
            .map_err(Error::ParseIntError)
    }

    pub fn address(&mut self) -> Result<u16, Error> {
        self.write_raw("ATMY\r")?;

        let resp = self.read_raw();

        u16::from_str_radix(&resp, 16)
            .map_err(Error::ParseIntError)
    }

    pub fn set_address(&mut self, addr: u16) -> Result<u16, Error> {
        self.write_raw(format!("ATMY{:x}\r", addr))?;

        let resp = self.read_raw();

        u16::from_str_radix(&resp, 16)
            .map_err(Error::ParseIntError)
    }

    pub fn dh(&mut self) -> Result<u16, Error> {
        self.write_raw("ATDH\r")?;

        let resp = self.read_raw();

        u16::from_str_radix(&resp, 16)
            .map_err(Error::ParseIntError)
    }

    pub fn set_dh(&mut self, dh: u16) -> Result<u16, Error> {
        self.write_raw(format!("ATDH{:x}\r", dh))?;

        let resp = self.read_raw();

        u16::from_str_radix(&resp, 16)
            .map_err(Error::ParseIntError)
    }

    pub fn dl(&mut self) -> Result<u16, Error> {
        self.write_raw("ATDL\r")?;

        let resp = self.read_raw();

        u16::from_str_radix(&resp, 16)
            .map_err(Error::ParseIntError)
    }

    pub fn set_dl(&mut self, dl: u16) -> Result<u16, Error> {
        self.write_raw(format!("ATDL{:x}\r", dl))?;

        let resp = self.read_raw();

        u16::from_str_radix(&resp, 16)
            .map_err(Error::ParseIntError)
    }

    pub fn edit_config<F: FnOnce(&mut XbeeConfig)>(&mut self, edit: F) {
        let mut config = XbeeConfig::new();

        edit(&mut config);

        if let Some(id) = config.id {
            self.set_id(id);
        }

        if let Some(addr) = config.addr {
            self.set_address(addr);
        }

        if let Some(dh) = config.dh {
            self.set_dh(dh);
        }

        if let Some(dl) = config.dl {
            self.set_dl(dl);
        }
    }
}

pub struct XbeeConfig {
    pub id: Option<u16>,
    pub addr: Option<u16>,
    pub dh: Option<u16>,
    pub dl: Option<u16>,
}

impl XbeeConfig {
    pub fn new() -> XbeeConfig {
        XbeeConfig {
            id: None,
            addr: None,
            dh: None,
            dl: None,
        }
    }

    pub fn set_id(&mut self, id: u16) -> &mut Self {
        self.id = Some(id);
        self
    }

    pub fn set_address(&mut self, addr: u16) -> &mut Self {
        self.addr = Some(addr);
        self
    }

    pub fn set_dh(&mut self, dh: u16) -> &mut Self {
        self.dh = Some(dh);
        self
    }

    pub fn set_dl(&mut self, dl: u16) -> &mut Self {
        self.dl = Some(dl);
        self
    }
}