extern crate byteorder;
#[macro_use] extern crate failure;
extern crate serialport;
extern crate xbee;

use byteorder::{ByteOrder, LittleEndian};
use failure::Error;
use xbee::*;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;
use serialport::SerialPortType;

#[derive(Debug, Fail)]
pub enum ConfigError {
    #[fail(display = "Not enough data given.")]
    NotEnoughData,
}

#[derive(Debug)]
struct Configuration {
    uuid: u32,
    min_voltage: f32,
    max_voltage: f32,
    min_value: f32,
    max_value: f32,
    name: String,
    units: String,
}

impl Configuration {
    fn new(raw: &[u8]) -> Result<Self, Error> {
        ensure!(raw.len() == 50, ConfigError::NotEnoughData);
        
        Ok(Configuration {
            uuid: LittleEndian::read_u32(&raw),
            min_voltage: LittleEndian::read_f32(&raw[4..]),
            max_voltage: LittleEndian::read_f32(&raw[8..]),
            min_value: LittleEndian::read_f32(&raw[12..]),
            max_value: LittleEndian::read_f32(&raw[16..]),
            name: String::from_utf8(raw[20..39].to_vec())
                .unwrap_or("".into())
                .replace("\x00", ""),
            units: String::from_utf8(raw[40..].to_vec())
                .unwrap_or("".into())
                .replace("\x00", ""),
        })
    }
}

fn main() {
    let port_name = match select_port() {
        Some(name) => name,
        None => {
            println!("Could not select port. Quitting.");
            return;
        }
    };

    let mut xbee = match Xbee::new(&port_name) {
        Ok(xbee) => xbee,
        Err(why) => {
            println!("Error: {}", why);
            return
        }
    };

    loop {
        println!("\nChoose a command:");
        println!("1. Get data");
        println!("2. Exit\n");

        let mut input = String::new();
        let stdin = std::io::stdin();

        stdin.lock()
            .read_line(&mut input)
            .expect("Could not read line");

        input = input.trim().into();

        if input == "1" {
            let mut tries = 0;
            loop {
                if tries >= 10 {
                    println!("Could not get data after 10 tries.");
                    break;
                }

                if let Err(why) = xbee.send_packet(2, b"C") {
                    println!("Could not send get request: {:?}", why);
                    continue;
                }

                if let Ok(packet) = xbee.read_packet() {
                    println!("{:?}", packet);
                    println!("{:?}", Configuration::new(&packet.data));
                    break;
                } else {
                    tries += 1;
                    thread::sleep(Duration::from_millis(100));
                }
            }
        } else if input == "2" {
            if let Err(why) = xbee.send_packet(2, b"B") {
                println!("Could not send get request: {:?}", why);
                continue;
            }
            if let Ok(packet) = xbee.read_packet() {
                println!("{:?}", packet);
                break;
            }
        } else {
            println!("Invalid selection: {}", input);
        }
    }
}

fn select_port() -> Option<String> {
    let ports = match serialport::available_ports() {
        Ok(ports) => ports,
        Err(why) => {
            println!("Could not enumerate serial ports: {:?}", why);
            return None;
        }
    };

    if ports.len() == 0 {
        println!("No serial ports found.");
        return None;
    }

    loop {
        println!("Select a port to connect to:");

        for (index, port) in ports.iter().enumerate() {
            if let SerialPortType::UsbPort(ref info) = port.port_type {
                println!("{}: {} - {}", index + 1, port.port_name, info.product.as_ref().map_or("", String::as_str));
            }
        }

        let stdin = std::io::stdin();
        let mut input = String::new();

        stdin.lock()
            .read_line(&mut input)
            .expect("Could not read from stdin.");

        let trimmed = input.trim();

        match trimmed.parse::<usize>() {
            Ok(num) if num > 0 && num <= ports.len() => {
                return Some(ports.get(num - 1).unwrap().port_name.clone())
            }
            _ => println!("Invalid selection."),
        }
    }
}