extern crate serial;

use serial::prelude::*;

use std::io::prelude::*;
use std::time::Duration;

fn main() {
    let mut port = match serial::open("COM6") {
        Ok(port) => port,
        Err(why) => {
            println!("Could not open port: {:?}", why);
            return;
        }
    };

    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud9600)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    });

    port.set_timeout(Duration::from_millis(1000));

    let mut buf = Vec::new();

    port.write(b"+++").expect("Could not write to serial");
    port.read(&mut buf[..]).expect("Could not read from serial.");

    println!("Got: {:?}", buf);
}