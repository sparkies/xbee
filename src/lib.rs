#![feature(io)]
extern crate byteorder;
#[macro_use] extern crate failure;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate parking_lot;
extern crate serial;

use failure::Error;
use serial::prelude::*;

use std::io::prelude::*;
use std::time::{Duration, Instant};
use std::ffi::OsStr;

pub mod packet;

use packet::*;

pub struct Xbee {
    port: serial::SystemPort,
    last_time: Instant,
}

impl Xbee {
    pub fn new<T: AsRef<OsStr> + ?Sized>(port: &T) -> Result<Xbee, Error> {
        let mut port = serial::open(port)?;

        port.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud9600)?;
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        })?;

        port.set_timeout(Duration::from_millis(10))?;

        Ok(Xbee {
            port: port,
            last_time: Instant::now(),
        })
    }

    pub fn write_raw(&mut self, data: &[u8]) -> Result<usize, Error> {
        let result = self.port.write(data)?;
        self.last_time = Instant::now();
        Ok(result)
    }

    pub fn read_raw(&mut self) -> String {
        let mut output = String::new();

        for _ in 1..300 {
            let mut data = [0; 1024];
            
            if let Ok(amount) = self.port.read(&mut data[..]) {
                output += std::str::from_utf8(&data[0..amount]).unwrap_or("");
            }
        }

        output
    }

    pub fn read_packet(&mut self) -> Result<Packet, Error> {
        let mut buffer = [0; 1024];
        let mut data = Vec::new();
        let mut length = 0;
        let mut tries = 0;
        
        loop {
            if let Ok(amount) = self.port.read(&mut buffer[..]) {
                data.extend_from_slice(&buffer[..amount]);
            }

            if length == 0 && data.len() >= 13 {
                length = data[12] as usize;
            } else if length > 0 && data.len() >= 13 + length {
                return Packet::from_data(&data);
            }

            ensure!(tries < 1000, PacketError::Timeout);

            tries += 1;
        }
    }

    pub fn send_packet(&mut self, dest: u32, data: &[u8]) -> Result<usize, Error> {
        let packet = Packet::new(dest, data);

        self.write_raw(&packet.as_bytes())
    }

    pub fn connect(&mut self) -> Result<bool, Error> {
        self.write_raw(b"+++")?;
        let resp = self.read_raw();

        Ok(resp == "OK\r")
    }

    pub fn connected(&self) -> bool {
        self.last_time.elapsed().as_secs() < 8
    }

    pub fn id(&mut self) -> Result<u16, Error> {
        self.write_raw(b"ATID\r")?;

        let mut resp = self.read_raw();
        resp.pop();

        let val = u16::from_str_radix(&resp, 16)?;
        Ok(val)
    }

    pub fn set_id(&mut self, id: u16) -> Result<bool, Error> {
        self.write_raw(format!("ATID{:x}\r", id).as_bytes())?;
        let resp = self.read_raw();
        Ok(resp == "OK\r")
    }

    pub fn address(&mut self) -> Result<u16, Error> {
        self.write_raw(b"ATMY\r")?;

        let mut resp = self.read_raw();
        resp.pop();

        let val = u16::from_str_radix(&resp, 16)?;
        Ok(val)
    }

    pub fn set_address(&mut self, addr: u16) -> Result<bool, Error> {
        self.write_raw(format!("ATMY{:x}\r", addr).as_bytes())?;
        let resp = self.read_raw();
        Ok(resp == "OK\r")
    }

    pub fn dh(&mut self) -> Result<u16, Error> {
        self.write_raw(b"ATDH\r")?;

        let mut resp = self.read_raw();
        resp.pop();

        let val = u16::from_str_radix(&resp, 16)?;
        Ok(val)
    }

    pub fn set_dh(&mut self, dh: u16) -> Result<bool, Error> {
        self.write_raw(format!("ATDH{:x}\r", dh).as_bytes())?;
        let resp = self.read_raw();
        Ok(resp == "OK\r")
    }

    pub fn dl(&mut self) -> Result<u16, Error> {
        self.write_raw(b"ATDL\r")?;

        let mut resp = self.read_raw();
        resp.pop();

        let val = u16::from_str_radix(&resp, 16)?;
        Ok(val)
    }

    pub fn set_dl(&mut self, dl: u16) -> Result<bool, Error> {
        self.write_raw(format!("ATDL{:x}\r", dl).as_bytes())?;
        let resp = self.read_raw();
        Ok(resp == "OK\r")
    }

    pub fn edit_config<F>(&mut self, edit: F) -> Result<(), Error> 
        where F: FnOnce(&mut XbeeConfig)
    {
        let mut config = XbeeConfig::new();

        edit(&mut config);

        if let Some(id) = config.id {
            self.set_id(id)?;
        }

        if let Some(addr) = config.addr {
            self.set_address(addr)?;
        }

        if let Some(dh) = config.dh {
            self.set_dh(dh)?;
        }

        if let Some(dl) = config.dl {
            self.set_dl(dl)?;
        }

        self.write_raw(b"ATWR")?;
        self.write_raw(b"ATAC")?;
        Ok(())
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