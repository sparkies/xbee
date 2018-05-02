extern crate serialport;
extern crate xbee;

use xbee::*;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;
use serialport::SerialPortType;

fn main() {    
    let port_name = match select_port() {
        Some(name) => name,
        None => {
            println!("Could not select port. Quitting.");
            return;
        }
    };

    let mut xbee = Xbee::new(&port_name).expect("Could not initialize Xbee");

    if let Err(why) = xbee.connect() {
        println!("Could not enter command mode: {:?}", why);
        return
    }

    xbee.edit_config(|ref mut c| {
        c.set_id(0x6789)
        .set_dh(0)
        .set_dl(0)
        .set_address(0);
    });

    loop {
        let mut cmd = String::new();
        let stdin = std::io::stdin();

        stdin.lock()
            .read_line(&mut cmd)
            .expect("Could not read line");

        cmd.pop();
        
        if cmd == "+++" {
            xbee.write_raw(cmd);
            thread::sleep(Duration::from_secs(3));
            println!("{}", xbee.read_raw());
        } 
        else {
            cmd.push('\r');
            xbee.write_raw(cmd);
            println!("{}", xbee.read_raw());
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