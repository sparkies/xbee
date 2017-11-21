#[macro_use] extern crate log;
extern crate env_logger;
extern crate serial_enumerate;
extern crate xbee;

use xbee::*;
use std::io::prelude::*;

fn main() {
    env_logger::init();

    let port_name = match select_port() {
        Some(name) => name,
        None => {
            println!("Could not select port. Quitting.");
            return;
        }
    };

    println!("Selected {}", port_name);

    let mut xbee = Xbee::new(&port_name).expect("Could not initialize Xbee");

    loop {
        let mut cmd = String::new();
        let stdin = std::io::stdin();

        stdin.lock()
            .read_line(&mut cmd)
            .expect("Could not read line");

        cmd = cmd.trim().into();
        
        if cmd == "+++" {
            if let Err(why) = xbee.connect() {
                println!("Could not enter command mode: {:?}", why);
                return
            }
            else {
                println!("Entered command mode.");
            }
        } 
        else {
            match &*cmd {
                "ATID" => println!("{:?}", xbee.id()),
                "ATDL" => println!("{:?}", xbee.dl()),
                "ATDH" => println!("{:?}", xbee.dh()),
                "ATMY" => println!("{:?}", xbee.address()),
                _ => {
                    xbee.send(cmd);
                }
            }
        }
    }
}

fn select_port() -> Option<String> {
    let ports = match serial_enumerate::enumerate_serial_ports() {
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

        for (index, device) in ports.iter().enumerate() {
            println!("{}: {}", index, device);
        }

        let stdin = std::io::stdin();
        let mut input = String::new();

        stdin.lock()
            .read_line(&mut input)
            .expect("Could not read from stdin.");

        let trimmed = input.trim();

        match trimmed.parse::<usize>() {
            Ok(num) => return Some(ports.get(num).unwrap().clone()),
            Err(why) => println!("Invalid selection."),
        }
    }
}