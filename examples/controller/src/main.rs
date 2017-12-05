extern crate pancurses;
extern crate serialport;
extern crate xbee;

use xbee::*;
use std::error::Error;
use std::thread;
use std::time::Duration;
use serialport::SerialPortType;

use pancurses::*;

const COLOR_TABLE: [i16; 8] = [COLOR_RED,
                                COLOR_BLUE,
                                COLOR_GREEN,
                                COLOR_CYAN,
                                COLOR_RED,
                                COLOR_MAGENTA,
                                COLOR_YELLOW,
                                COLOR_WHITE];

fn main() {
    let mut window = initscr();
    let current = window.subwin(5, 28, 5, 41).expect("Could not make subwindow");

    let title = "Sparkies Demo";
    
    window.nodelay(true);
    noecho();

    curs_set(0);
    window.keypad(true);

    for (i, color) in COLOR_TABLE.into_iter().enumerate() {
        init_pair(i as i16, *color, COLOR_BLACK);
    }

    window.clear();
    window.mvprintw(0, (window.get_max_x() / 2) - ((title.len() / 2) as i32), title);
    window.refresh();

    let port_name = match select_port(&mut window) {
        Some(name) => name,
        None => {
            window.mvprintw(window.get_max_y() - 1, 0, "Could not select port. Quitting.");
            return;
        }
    };

    let mut xbee = match Xbee::new(&port_name) {
        Ok(xbee) => xbee,
        Err(why) => {
            match why {
                xbee::Error::SerialError(err) => {
                    window.mvprintw(window.get_max_y() - 1, 0, &format!("Serial error: {}", err.description()));
                }
                _ => {
                    window.mvprintw(window.get_max_y() - 1, 0, "Other error.");
                    ()
                }
            }
            return
        }
    };

    window.mv(1, 0);
    window.clrtobot();

    window.mvprintw(5, 10, "Available commands:");
    window.mvprintw(6, 10, "1. Set threshold");
    window.mvprintw(7, 10, "2. Exit\n");
    window.refresh();

    loop {
        match window.getch() {
            Some(Input::Character('1')) => {
                let mut value = String::new();

                loop {
                    window.mvprintw(window.get_max_y() - 1, 0, "Enter new threshold: ");
                    window.clrtobot();

                    echo();
                    loop {
                        if let Some(Input::Character(c)) = window.getch() {
                            if c == '\n' {
                                break;
                            }
                            value.push(c);
                        }
                    }
                    noecho();

                    value = value.trim().into();

                    if let Ok(_) = value.parse::<u32>() {
                        break;
                    } else {
                        value.clear();
                    }
                }

                let mut tries = 0;

                loop {
                    if tries >= 10 {
                        window.mvprintw(window.get_max_y() - 1, 0, "Could not set after 10 tries.");
                        break;
                    }

                    if let Err(why) = xbee.write_raw(format!("set {}", value)) {
                        window.mvprintw(window.get_max_y() - 1, 0, &format!("Could not send set request: {:?}", why));
                    }

                    let mut resp = xbee.read_raw();
                    resp = resp.trim().into();
                    
                    if resp == "OK" {
                        window.mvprintw(window.get_max_y() - 1, 0, &format!("New threshold set to {}", value));
                        break;
                    } else {
                        tries += 1;
                    }

                    thread::sleep(Duration::from_millis(100));
                }
            }
            Some(Input::Character('2')) => {
                break;
            }
            _ => (),
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

        if values.len() == 3 {
            let title = "Current data";
            current.clear();
            current.draw_box(0, 0);
            current.mvprintw(0, (current.get_max_x() / 2) - ((title.len() / 2) as i32), title);
            current.mvprintw(1, 1, &format!("Temperature: {} C", values[0]));
            current.mvprintw(2, 1, &format!("Humidity   : {}%", values[1]));
            current.mvprintw(3, 1, &format!("Threshold  : {}", values[2]));
            current.refresh();
        }
    }
}

fn select_port(window: &mut Window) -> Option<String> {
    let ports = match serialport::available_ports() {
        Ok(ports) => ports,
        Err(why) => {
            window.mvprintw(window.get_max_y() - 1, 0, &format!("Could not enumerate serial ports: {:?}", why));
            window.refresh();
            return None;
        }
    };

    if ports.len() == 0 {
        window.mvprintw(window.get_max_y() - 1, 0, "No serial ports found.");
        window.refresh();
        return None;
    }

    let prompt = "Select a port to connect to:";

    loop {
        window.mvprintw(1, (window.get_max_x() / 2) - ((prompt.len() / 2) as i32), prompt);

        for (index, port) in ports.iter().enumerate() {
            if let SerialPortType::UsbPort(ref info) = port.port_type {
                window.mvprintw(index as i32 + 2, 10, &format!("{}: {} - {}", index + 1, port.port_name, info.product.as_ref().map_or("", String::as_str)));
            }
        }
        window.refresh();

        let mut input = String::new();
        loop {
            match window.getch() {
                Some(Input::Character(c)) => {
                    if c == '\n' {
                        break;
                    }

                    input.push(c);
                }
                _ => (),
            }
        }

        let trimmed = input.trim();

        match trimmed.parse::<usize>() {
            Ok(num) if num > 0 && num <= ports.len() => {
                return Some(ports.get(num - 1).unwrap().port_name.clone())
            }
            _ => {
                window.mvprintw(window.get_max_y() - 1, 0, "Invalid selection.");
                ()
            }
        }
    }
}