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

    port.set_timeout(Duration::from_millis(2000));

    let mut buf = String::new();

    port.write(b"+++");
    port.read_to_string(&mut buf);
    println!("Response: '{:?}'", buf);

    while buf != "ATCN\r" {
        buf = "".into();
        let mut cmd = String::new();
        let stdin = std::io::stdin();
        stdin.lock()
            .read_line(&mut cmd)
            .expect("Could not read line");
        cmd += "\r".into();
        port.write(&cmd.as_bytes());
        port.read_to_string(&mut buf);
        println!("Response: '{:?}'", buf);
    }
}