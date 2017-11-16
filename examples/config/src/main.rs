extern crate xbee;

use xbee::*;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;

fn main() {
    let mut xbee = Xbee::new("COM6").expect("Could not initialize Xbee");

    xbee.write_raw("+++");
    println!("{}", xbee.read_raw());

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