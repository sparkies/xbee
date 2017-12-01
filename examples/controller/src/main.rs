extern crate serialport;
extern crate xbee;

use xbee::*;
use std::io::prelude::*;
use std::error::Error;
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

    let mut xbee = match Xbee::new(&port_name) {
        Ok(xbee) => xbee,
        Err(why) => {
            match why {
                xbee::Error::SerialError(err) => {
                    println!("Serial error: {}", err.description());
                }
                _ => println!("Other"),
            }
            return
        }
    };

    loop {
        println!("\nChoose a command:");
        println!("1. Set threshold");
        println!("2. Get data");
        println!("3. Exit\n");

        let mut input = String::new();
        let stdin = std::io::stdin();

        stdin.lock()
            .read_line(&mut input)
            .expect("Could not read line");

        input = input.trim().into();

        if input == "1" {
            let mut value = String::new();

            loop {
                print!("Enter new threshold: ");
                let _ = std::io::stdout().flush();

                stdin.lock()
                    .read_line(&mut value)
                    .expect("Could not read line");

                value = value.trim().into();

                if let Ok(_) = value.parse::<u32>() {
                    break;
                }
            }

            let mut tries = 0;

            loop {
                if tries >= 10 {
                    println!("Could not set after 10 tries.");
                    break;
                }

                if let Err(why) = xbee.write_raw(format!("set {}", value)) {
                    println!("Could not send set request: {:?}", why);
                }

                let mut resp = xbee.read_raw();
                resp = resp.trim().into();
                
                if resp == "OK" {
                    println!("New threshold set to {}", value);
                    break;
                } else {
                    tries += 1;
                }

                thread::sleep(Duration::from_millis(100));
            }
        } 
        else if input == "2" {
            let mut tries = 0;
            loop {
                if tries >= 10 {
                    println!("Could not get data after 10 tries.");
                    break;
                }

                if let Err(why) = xbee.write_raw("get") {
                    println!("Could not send get request: {:?}", why);
                    continue;
                }

                let resp = xbee.read_raw();

                let values = resp
                    .split_whitespace()
                    .filter_map(|v| v.parse::<f64>().ok())
                    .collect::<Vec<f64>>();

                if values.len() < 3 {
                    tries += 1;
                } else {
                    println!("Temperature: {}°C", values[0]);
                    println!("Humidity   : {}%", values[1]);
                    println!("Threshold : {}", values[2]);
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
        } else if input == "3" {
            break;
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