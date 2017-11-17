extern crate xbee;

use xbee::*;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;

fn main() {
    let mut xbee = Xbee::new("COM6").expect("Could not initialize Xbee");

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