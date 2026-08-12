mod initialization;
mod instruction;
mod memory;
mod random;
mod screen;
mod spin_sleep;
mod virtual_machine;

use std::{
    env::Args,
    io::Write,
    path::Path,
    print, println,
    str::FromStr,
    sync::{Arc, Mutex},
    thread::spawn,
    time::{Duration, Instant},
};

use minifb::{Icon, Key};

use crate::{screen::Screen, spin_sleep::sleep};

const DURATION_1_SECOND: Duration = Duration::from_secs(1);
const UNDER_RECOMMENDED_FREQUENCY_HZ: u16 = 1990;
const TICK_RATE: u8 = 60;
const DURATION_FRAME_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / TICK_RATE as u64);

struct Arguments {
    should_exit: bool,
    executable_file_path: Option<String>,
    running_at_frequency_hz: Option<u32>,
    show_information: bool,
    no_vertical_synchronization: bool,
}

impl Arguments {
    pub fn parse() -> Self {
        let mut should_exit: bool = false;
        let mut is_help: bool = false;
        let mut arguments: Args = std::env::args();
        let path_string: String = arguments.next().unwrap();
        let path: &Path = Path::new(&path_string);
        let mut executable_file_path: Option<String> = None;
        let mut running_at_frequency_hz: Option<u32> = None;
        let mut show_information: bool = false;
        let mut no_vertical_synchronization: bool = false;

        // bit0: should_parse_executable, bit1: is_parsed_executable, bit2: should_parse_freq, bit3: is_parsed_freq
        let mut temp: u8 = 0;

        macro_rules! error_exit {
            ($($arg:tt)*) => {
                println!($($arg)*);
                should_exit = true;
                break;
            };
        }

        while let Some(argument) = arguments.next() {
            if argument == "--help" {
                println!(
                    "\x1b[0mA CHIP-8 virtual machine, running programs

\x1b[1;32mUsage: \x1b[1;96m{} \x1b[0;36m[OPTIONS]

\x1b[1;32mOptions:
  \x1b[1;96m-r\x1b[0m, \x1b[1;96m--run \x1b[0;36m<PATH>
      \x1b[0mThe path of the program to run

  \x1b[1;96m-f\x1b[0m, \x1b[1;96m--frequency \x1b[0;36m<FREQUENCY_HZ>
      \x1b[0mThe main frequency at which Tuff Virtual Machine will run. Leave empty to run at frequency of 500Hz by default. Type '\x1b[1;33m0\x1b[0m' to enable unlimited mode (unstable)

  \x1b[1;96m-i\x1b[0m, \x1b[1;96m--with-debug-information
      \x1b[0mShow debug information

      \x1b[1;96m--no-vertical-synchronization
      \x1b[0mDisable 60Hz frame synchronization. By default, the VM waits for the render thread

  \x1b[1;96m-h\x1b[0m, \x1b[1;96m--help
      \x1b[0mPrint help (see a summary with '\x1b[1;33m-h\x1b[0m')

  \x1b[1;96m-V\x1b[0m, \x1b[1;96m--version
      \x1b[0mPrint version\x1b[0m",
                    path.file_name().unwrap().to_str().unwrap()
                );
                should_exit = true;
                is_help = true;
                break;
            } else if argument == "--version" {
                println!("\x1b[0mTuff Virtual Machine 0.1.0");
                should_exit = true;
                is_help = true;
                break;
            } else if argument == "--frequency" {
                if temp & 0b0000_1000 != 0 {
                    error_exit!(
                        "\x1b[1;31merror: \x1b[0mthe argument '\x1b[1;33m--frequency <FREQUENCY_HZ>\x1b[0m' cannot be used multiple times"
                    );
                };
                if let Some(value) = arguments.next() {
                    if let Ok(num) = value.parse::<u32>() {
                        running_at_frequency_hz = Some(num);
                        temp |= 0b0000_1000;
                    } else {
                        error_exit!(
                            "\x1b[1;31merror: \x1b[0minvalid value '\x1b[1;33m{}\x1b[0m' for '\x1b[1;33m--frequency <FREQUENCY_HZ>\x1b[0m': number too large to fit in target type",
                            value
                        );
                    };
                } else {
                    error_exit!(
                        "\x1b[1;31merror: \x1b[0ma value is required for '\x1b[1;33m--frequency <FREQUENCY_HZ>\x1b[0m' but none was supplied"
                    );
                };
            } else if argument == "--run" {
                if temp & 0b0000_0010 != 0 {
                    error_exit!(
                        "\x1b[1;31merror: \x1b[0mthe argument '\x1b[1;33m--run <PATH>\x1b[0m' cannot be used multiple times"
                    );
                };
                if let Some(value) = arguments.next() {
                    executable_file_path = Some(value);
                    temp |= 0b0000_0010;
                } else {
                    error_exit!(
                        "\x1b[1;31merror: \x1b[0ma value is required for '\x1b[1;33m--run <PATH>\x1b[0m' but none was supplied"
                    );
                };
            } else if argument == "--with-debug-information" {
                if show_information {
                    error_exit!(
                        "\x1b[1;31merror: \x1b[0mthe argument '\x1b[1;33m--with-debug-information\x1b[0m' cannot be used multiple times"
                    );
                };
                show_information = true;
            } else if argument == "--no-vertical-synchronization" {
                if no_vertical_synchronization {
                    error_exit!(
                        "\x1b[1;31merror: \x1b[0mthe argument '\x1b[1;33m--no-vertical-synchronization\x1b[0m' cannot be used multiple times"
                    );
                };
                no_vertical_synchronization = true;
            } else if argument.starts_with('-') && !argument.starts_with("--") {
                let rest: &str = &argument[1..];
                let characters: Vec<char> = rest.chars().collect();
                let mut i: usize = 0;
                while i < characters.len() {
                    let character: char = characters[i];
                    match character {
                        'h' => {
                            println!(
                                "\x1b[0mRuns CHIP-8 programs

\x1b[1;32mUsage: \x1b[1;96m{} \x1b[0;36m[OPTIONS]

\x1b[1;32mOptions:
  \x1b[1;96m-r\x1b[0m, \x1b[1;96m--run \x1b[0;36m<PATH>                   \x1b[0mThe path of the program to run
  \x1b[1;96m-f\x1b[0m, \x1b[1;96m--frequency \x1b[0;36m<FREQUENCY_HZ>     \x1b[0mThe main frequency at which Tuff Virtual Machine will run
  \x1b[1;96m-i\x1b[0m, \x1b[1;96m--with-debug-information       \x1b[0mShow debug information
      \x1b[1;96m--no-vertical-synchronization  \x1b[0mDisable 60Hz frame synchronization
  \x1b[1;96m-h\x1b[0m, \x1b[1;96m--help                         \x1b[0mPrint help (see more with '\x1b[1;33m--help\x1b[0m')
  \x1b[1;96m-V\x1b[0m, \x1b[1;96m--version                      \x1b[0mPrint version\x1b[0m",
                                path.file_name().unwrap().to_str().unwrap()
                            );
                            should_exit = true;
                            is_help = true;
                            break;
                        }
                        'V' => {
                            println!("\x1b[0mTuff Virtual Machine 0.1.0\x1b[0m");
                            should_exit = true;
                            is_help = true;
                            break;
                        }
                        'i' => {
                            if show_information {
                                error_exit!(
                                    "\x1b[1;31merror: \x1b[0mthe argument '\x1b[1;33m-i\x1b[0m' cannot be used multiple times"
                                );
                            };
                            show_information = true;
                        }
                        'f' => {
                            if temp & 0b0000_1000 != 0 {
                                error_exit!(
                                    "\x1b[1;31merror: \x1b[0mthe argument '\x1b[1;33m-f <FREQUENCY_HZ>\x1b[0m' cannot be used multiple times"
                                );
                            };
                            if i + 1 < characters.len() {
                                let value_string: String = characters[i + 1..].iter().collect();
                                if let Ok(number) = value_string.parse::<u32>() {
                                    running_at_frequency_hz = Some(number);
                                    temp |= 0b0000_1000;
                                    i = characters.len();
                                } else {
                                    error_exit!(
                                        "\x1b[1;31merror: \x1b[0minvalid value '\x1b[1;33m{}\x1b[0m' for '\x1b[1;33m-f <FREQUENCY_HZ>\x1b[0m': number too large to fit in target type",
                                        value_string
                                    );
                                };
                            } else {
                                if let Some(value) = arguments.next() {
                                    if let Ok(number) = value.parse::<u32>() {
                                        running_at_frequency_hz = Some(number);
                                        temp |= 0b0000_1000;
                                    } else {
                                        error_exit!(
                                            "\x1b[1;31merror: \x1b[0minvalid value '\x1b[1;33m{}\x1b[0m' for '\x1b[1;33m-f <FREQUENCY_HZ>\x1b[0m': number too large to fit in target type",
                                            value
                                        );
                                    };
                                } else {
                                    error_exit!(
                                        "\x1b[1;31merror: \x1b[0ma value is required for '\x1b[1;33m-f <FREQUENCY_HZ>\x1b[0m' but none was supplied"
                                    );
                                }
                            }
                        }
                        'r' => {
                            if temp & 0b0000_0010 != 0 {
                                error_exit!(
                                    "\x1b[1;31merror: \x1b[0mthe argument '\x1b[1;33m-r <PATH>\x1b[0m' cannot be used multiple times"
                                );
                            };
                            if i + 1 < characters.len() {
                                let value_string: String = characters[i + 1..].iter().collect();
                                if value_string.starts_with('-') {
                                    error_exit!(
                                        "\x1b[1;31merror: \x1b[0munexpected argument '\x1b[1;33m{}\x1b[0m' found",
                                        value_string
                                    );
                                };
                                executable_file_path = Some(value_string);
                                temp |= 0b0000_0010;
                                i = characters.len();
                            } else {
                                if let Some(value) = arguments.next() {
                                    executable_file_path = Some(value);
                                    temp |= 0b0000_0010;
                                } else {
                                    error_exit!(
                                        "\x1b[1;31merror: \x1b[0ma value is required for '\x1b[1;33m-r <PATH>\x1b[0m' but none was supplied"
                                    );
                                };
                            };
                        }
                        _ => {
                            error_exit!(
                                "\x1b[1;31merror: \x1b[0munexpected argument '\x1b[1;33m-{}\x1b[0m' found",
                                character
                            );
                        }
                    };
                    if should_exit || is_help {
                        break;
                    };
                    i += 1;
                }
            } else if argument == "--" {
                break;
            } else {
                error_exit!(
                    "\x1b[1;31merror: \x1b[0munexpected argument '\x1b[1;33m{}\x1b[0m' found",
                    argument
                );
            };

            if should_exit || is_help {
                break;
            };
        }

        if should_exit && !is_help {
            println!(
                "\n\x1b[1;32mUsage: \x1b[1;36m{} \x1b[36m[OPTIONS]\n\n\x1b[0mFor more information, try '\x1b[1;36m--help\x1b[0m'.",
                path.file_name().unwrap().to_str().unwrap()
            );
        };

        Self {
            should_exit,
            executable_file_path,
            running_at_frequency_hz,
            show_information,
            no_vertical_synchronization,
        }
    }
}

fn main() {
    let argument: Arguments = Arguments::parse();

    if argument.should_exit {
        return;
    };

    print!("\x1b[2J\x1b[1;1H");
    std::io::stdout().flush().unwrap();

    let running_at_frequency_hz: u32 =
        *argument.running_at_frequency_hz.as_ref().unwrap_or(&500u32);
    let rom_file_path: Option<&Path> = if let Some(p) = argument.executable_file_path.as_ref() {
        Some(&Path::new(p))
    } else {
        None
    };
    let show_information: bool = argument.show_information;
    let no_vertical_synchronization: bool = argument.no_vertical_synchronization;

    let mut information_head: &str = "";
    if show_information {
        information_head = "\r\x1b[KWelcome to Tuff Virtual Machine!\n";
    } else {
        println!(
            "\r\x1b[KWelcome to Tuff Virtual Machine!\n{}{}",
            if no_vertical_synchronization
                && (440 > running_at_frequency_hz || running_at_frequency_hz > 520)
            {
                "\r\x1b[KNote: To simulate vertical synchronization, run Tuff Virtual Machine at frequency of around 500Hz.\n"
            } else {
                ""
            },
            if running_at_frequency_hz == 0
                || running_at_frequency_hz >= UNDER_RECOMMENDED_FREQUENCY_HZ as u32
            {
                format!(
                    "\r\x1b[KWarning: Running at high frequency (above {}KHz) may bring instability, high power consumption, and program crashes!\n\r\x1b[KNote: Recommended frequency is below {}Hz for optimal stability.\n",
                    UNDER_RECOMMENDED_FREQUENCY_HZ, UNDER_RECOMMENDED_FREQUENCY_HZ
                )
            } else {
                "".to_string()
            }
        );
    };

    let ((speed_tx, mut virtual_machine_instance), mut window) = initialization::initialize(
        running_at_frequency_hz,
        rom_file_path,
        no_vertical_synchronization,
    );

    let current_key: Arc<Mutex<[bool; 16]>> = Arc::new(Mutex::new([false; 16]));
    let current_key_clone: Arc<Mutex<[bool; 16]>> = current_key.clone();

    let screen: Arc<Mutex<Screen>> = Arc::new(Mutex::new(Screen::new()));
    let screen_clone: Arc<Mutex<Screen>> = screen.clone();

    let mut screen_buffer: [u32; 64 * 32] = [0; 64 * 32];
    let mut next_frame: Instant = Instant::now() + DURATION_FRAME_INTERVAL;
    spawn(move || {
        let deviation: i64 = (running_at_frequency_hz as f32 * 0.015).round() as i64;
        let mut last_second: Instant = Instant::now();
        let mut tick_counter: u32 = 0;
        let mut real_frequency_hz: u32 = 0;
        let mut virtual_machine_status_snapshot: ([u8; 16], [u16; 16], usize, u16, usize, u8, u8);
        loop {
            if let Ok(keys) = current_key_clone.try_lock() {
                virtual_machine_instance.set_current_key(&keys);
            };

            if let Ok(mut s) = screen.try_lock() {
                *s = virtual_machine_instance.get_screen().clone();
            };

            if virtual_machine_instance.tick() {
                tick_counter += 1;
            };

            if Instant::now() - last_second > DURATION_1_SECOND {
                if no_vertical_synchronization && running_at_frequency_hz != 0 {
                    if (tick_counter as i64 - running_at_frequency_hz as i64) < -deviation {
                        let _: Result<(), std::sync::mpsc::SendError<virtual_machine::Speed>> =
                            speed_tx.send(crate::virtual_machine::Speed::SpeedUp);
                    } else if (tick_counter as i64 - running_at_frequency_hz as i64) > deviation {
                        let _: Result<(), std::sync::mpsc::SendError<virtual_machine::Speed>> =
                            speed_tx.send(crate::virtual_machine::Speed::SlowDown);
                    };
                };
                real_frequency_hz = tick_counter;
                if !show_information {
                    print!("\r\x1b[KReal Main Frequency: {}Hz", real_frequency_hz);
                    std::io::stdout().flush().unwrap();
                };
                last_second = Instant::now();
                tick_counter = 0;
            };

            virtual_machine_status_snapshot = virtual_machine_instance.get_register();

            if show_information {
                let information: String = format!(
                    "\x1b[1;1H{}\x1b[KReal Main Frequency: {}Hz\n\r\x1b[KRegisters (v0x0-v0xF):\n{}\n\r\x1b[KCall Stack (with Stack Pointer at 0x{:01X}, Old -> New Order):\n{}\n\r\x1b[KI = 0x{:03X}, PC = 0x{:03X}, DT = 0x{:02X}, ST = 0x{:02X}\n\x1b[J",
                    information_head,
                    real_frequency_hz,
                    virtual_machine_status_snapshot
                        .0
                        .iter()
                        .map(|&x| format!("0x{:02X}", x))
                        .collect::<Vec<String>>()
                        .join(" "),
                    virtual_machine_status_snapshot.2,
                    virtual_machine_status_snapshot
                        .1
                        .iter()
                        .map(|&x| format!("0x{:03X}", x))
                        .collect::<Vec<String>>()
                        .join(" "),
                    virtual_machine_status_snapshot.3,
                    virtual_machine_status_snapshot.4,
                    virtual_machine_status_snapshot.5,
                    virtual_machine_status_snapshot.6
                );
                print!("{}", information);
            };
        }
    });

    window.set_icon(Icon::from_str("assets/icon.ico").unwrap());
    while window.is_open() {
        if let Ok(mut keys) = current_key.lock() {
            keys[0x0] = window.is_key_down(Key::X);
            keys[0x1] = window.is_key_down(Key::Key1);
            keys[0x2] = window.is_key_down(Key::Key2);
            keys[0x3] = window.is_key_down(Key::Key3);
            keys[0x4] = window.is_key_down(Key::Q);
            keys[0x5] = window.is_key_down(Key::W);
            keys[0x6] = window.is_key_down(Key::E);
            keys[0x7] = window.is_key_down(Key::A);
            keys[0x8] = window.is_key_down(Key::S);
            keys[0x9] = window.is_key_down(Key::D);
            keys[0xA] = window.is_key_down(Key::Z);
            keys[0xB] = window.is_key_down(Key::C);
            keys[0xC] = window.is_key_down(Key::Key4);
            keys[0xD] = window.is_key_down(Key::R);
            keys[0xE] = window.is_key_down(Key::F);
            keys[0xF] = window.is_key_down(Key::V);
        };

        if let Ok(s) = screen_clone.lock() {
            screen_buffer = {
                let original_buffer: [bool; 64 * 32] = s.get_buffer();
                let mut converted_buffer: [u32; 64 * 32] = [0; 64 * 32];
                for (i, pixel) in original_buffer.iter().enumerate() {
                    converted_buffer[i] = (*pixel as u32) * 0xFFFFFF;
                }
                converted_buffer
            };
            s.set_drew();
        };

        let _: Result<(), minifb::Error> = window.update_with_buffer(&screen_buffer, 64, 32);
        let now: Instant = Instant::now();
        if !no_vertical_synchronization {
            if now < next_frame {
                sleep(next_frame - now);
            };
            next_frame += DURATION_FRAME_INTERVAL;
        };
    }

    println!("\nQuit.");
    std::process::exit(0);
}
