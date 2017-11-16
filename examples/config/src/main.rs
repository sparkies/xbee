extern crate xbee;

use xbee::*;
use std::io::prelude::*;

fn main() {
    let mut xbee = Xbee::new("COM6").expect("Could not initialize Xbee");


    let mut buf = String::new();

    match xbee.write_raw("+++") {
        Ok(size) => println!("Wrote {}", size),
        Err(why) => println!("Error writing: {:?}", why),
    }

    buf = xbee.read_raw();

    println!("Response: '{:?}'", buf);

    loop {
        let mut cmd = String::new();
        let stdin = std::io::stdin();

        stdin.lock()
            .read_line(&mut cmd)
            .expect("Could not read line");

        if cmd != "+++" {
            cmd += "\r".into();
        }

        xbee.write_raw(cmd);

        println!("{}", xbee.read_raw());
    }
}