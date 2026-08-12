use std::{
    cell::RefMut,
    f32::consts::PI,
    path::Path,
    process::exit,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};

use cpal::{
    Device, Host, Stream, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::{
    instruction::Opcode, memory::RAM, random::TinyMT8Bit, screen::Screen, spin_sleep::sleep,
};

const FREQUENCY_60HZ: Duration = Duration::from_nanos(1_000_000_000 / 60);
const DURATION_50NS: Duration = Duration::from_nanos(50);
const DURATION_1_000NS: Duration = Duration::from_nanos(1_000);

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub enum Speed {
    SpeedUp,
    SlowDown,
}

struct Clock {
    receiver: mpsc::Receiver<()>,
}

impl Clock {
    pub fn new(frequency_hz: u32) -> (mpsc::Sender<Speed>, Self) {
        let (tx, rx) = mpsc::channel();
        let clock: Clock = Clock { receiver: rx };

        let (speed_tx, speed_rx) = mpsc::channel::<Speed>();

        thread::spawn(move || {
            if frequency_hz == 0 {
                loop {
                    let _: Result<(), mpsc::SendError<()>> = tx.send(());
                }
            } else if frequency_hz < 2_000 {
                let frequency_ns: u32 = 1_000_000_000 as u32 / frequency_hz;
                let mut duration: Duration = Duration::new(0, frequency_ns - 1_000); // Relatively precise
                loop {
                    sleep(duration);
                    if let Ok(action) = speed_rx.try_recv() {
                        match action {
                            Speed::SpeedUp => {
                                duration -= if duration >= DURATION_1_000NS {
                                    DURATION_1_000NS
                                } else {
                                    duration
                                };
                            }
                            Speed::SlowDown => {
                                duration += DURATION_50NS;
                            }
                        };
                    };
                    let _: Result<(), mpsc::SendError<()>> = tx.send(());
                }
            } else {
                let frequency_ns: u32 = 1_000_000_000 as u32 / frequency_hz;
                let mut duration: Duration = Duration::new(0, frequency_ns);
                let mut last_sleep: Instant = Instant::now();
                loop {
                    while Instant::now() - last_sleep <= duration {
                        if let Ok(action) = speed_rx.try_recv() {
                            match action {
                                Speed::SpeedUp => {
                                    duration -= if duration >= DURATION_50NS {
                                        DURATION_50NS
                                    } else {
                                        duration
                                    };
                                }
                                Speed::SlowDown => {
                                    duration += DURATION_50NS;
                                }
                            };
                        };
                    }
                    last_sleep = Instant::now();
                    let _: Result<(), mpsc::SendError<()>> = tx.send(());
                }
            };
        });

        (speed_tx, clock)
    }
}

pub struct AudioPlayer {
    _stream: Stream,
    playing: Arc<Mutex<bool>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let host: Host = cpal::default_host();
        let device: Device = host.default_output_device().expect("no output device");
        let config: SupportedStreamConfig = device.default_output_config().unwrap();
        let sample_rate: u32 = config.sample_rate();
        let channels: u16 = config.channels();
        let playing: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
        let playing_clone: Arc<Mutex<bool>> = playing.clone();

        let _stream: Stream = device
            .build_output_stream(
                config.config(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let play: bool = *playing_clone.lock().unwrap();
                    if play {
                        let phase_increment: f32 = 440.0 / sample_rate as f32;
                        thread_local! {
                            static PHASE: std::cell::RefCell<f32> = std::cell::RefCell::new(0.0);
                        };
                        PHASE.with(|phase_cell| {
                            let mut phase: RefMut<'_, f32> = phase_cell.borrow_mut();
                            for frame in data.chunks_mut(channels.into()) {
                                let sample: f32 = (*phase * 2.0 * PI).sin() * 0.3;

                                *phase = (*phase + phase_increment) % 1.0;
                                for sample_out in frame.iter_mut() {
                                    *sample_out = sample;
                                }
                            }
                        });
                    } else {
                        for sample in data.iter_mut() {
                            *sample = 0.0;
                        }
                    };
                },
                |err| eprintln!("audio error: {}", err),
                None,
            )
            .unwrap();
        _stream.play().unwrap();
        Self { _stream, playing }
    }

    pub fn set_playing(self: &mut Self, play: bool) {
        *self.playing.lock().unwrap() = play;
    }
}

pub struct VirtualMachine {
    running_at_frequency_hz: u32,
    no_vertical_synchronization: bool,
    current_keys: [bool; 16],
    clock: Clock,
    program_counter: usize,
    ram: RAM<4096>,
    screen: Screen,
    stack: [u16; 16],
    stack_pointer: usize,
    registers: [u8; 16],
    index_register: u16,
    delay_timer: u8,
    sound_timer: u8,
    update_receiver: mpsc::Receiver<()>,
    wait_key_released: bool,
    last_tick: Instant,
    last_update: Instant,
    random_number_generator: TinyMT8Bit,
    audio_player: AudioPlayer,
    was_playing: bool,
}

impl VirtualMachine {
    pub fn new(
        frequency_hz: u32,
        rom_file_path: Option<&Path>,
        program_to_load: Option<&[u8]>,
        no_vertical_synchronization: bool,
    ) -> (mpsc::Sender<Speed>, Self) {
        {
            let (tx, rx) = mpsc::channel();
            let (speed_tx, clock) = Clock::new(frequency_hz);
            let virtual_machine_instance: VirtualMachine = VirtualMachine {
                running_at_frequency_hz: frequency_hz,
                no_vertical_synchronization: no_vertical_synchronization,
                current_keys: [false; 16],
                clock: clock,
                program_counter: 0x200,
                ram: if let Some(path) = rom_file_path {
                    let mut ram: RAM<4096> = RAM::new();
                    ram.load_from_file(path, 0x200);
                    ram
                } else if let Some(content) = program_to_load {
                    let mut ram: RAM<4096> = RAM::new();
                    ram.load_from_array(content, 0x200);
                    ram
                } else {
                    RAM::new()
                },
                screen: Screen::new(),
                stack: [0; 16],
                stack_pointer: 0,
                registers: [0; 16],
                index_register: 0,
                delay_timer: 0,
                sound_timer: 0,
                update_receiver: rx,
                wait_key_released: false,
                last_tick: Instant::now(),
                last_update: Instant::now(),
                random_number_generator: TinyMT8Bit::new(),
                audio_player: AudioPlayer::new(),
                was_playing: false,
            };
            thread::spawn(move || {
                loop {
                    sleep(FREQUENCY_60HZ);
                    let _: Result<(), mpsc::SendError<()>> = tx.send(());
                }
            });
            (speed_tx, virtual_machine_instance)
        }
    }

    pub fn set_font(self: &mut Self, font: Option<[u8; 80]>) -> () {
        match font {
            Some(f) => self.ram.load_from_array(&f, 0x0050),
            None => self.ram.load_from_array(&FONT, 0x0050),
        };
    }

    pub fn set_current_key(self: &mut Self, keys: &[bool; 16]) -> () {
        self.current_keys = *keys;
    }

    pub fn get_screen(self: &Self) -> &Screen {
        &self.screen
    }

    pub fn get_register(self: &Self) -> ([u8; 16], [u16; 16], usize, u16, usize, u8, u8) {
        (
            self.registers,
            self.stack,
            self.stack_pointer,
            self.index_register,
            self.program_counter,
            self.delay_timer,
            self.sound_timer,
        )
    }

    pub fn tick(self: &mut Self) -> bool {
        match self.update_receiver.try_recv() {
            Ok(_) => {
                self.update_timer();
                self.last_update = Instant::now();
            }
            Err(_) => {
                if self.running_at_frequency_hz != 0 && self.running_at_frequency_hz < 1990 {
                    let now: Instant = Instant::now();
                    let tick_duration: Duration =
                        Duration::from_nanos((1_000_000_000 / self.running_at_frequency_hz) as u64);
                    if !(now - self.last_tick >= tick_duration)
                        && !(now - self.last_update >= FREQUENCY_60HZ)
                    {
                        let remain_tick: Duration = tick_duration - (now - self.last_tick);
                        let remain_update: Duration = FREQUENCY_60HZ - (now - self.last_update);
                        sleep(remain_tick.min(remain_update));
                    };
                };
            }
        };
        let should_play: bool = self.sound_timer > 0;
        if should_play != self.was_playing {
            self.audio_player.set_playing(should_play);
            self.was_playing = should_play;
        };
        match self.clock.receiver.try_recv() {
            Ok(_) => {}
            Err(_) => {
                return false;
            }
        };
        self.step();
        self.last_tick = Instant::now();
        true
    }

    #[inline(always)]
    fn update_timer(self: &mut Self) -> () {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        };
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        };
    }

    fn step(self: &mut Self) -> () {
        let instruction: u16 = self.fetch();
        let instruction_decoded: Opcode = self.decode(instruction);
        self.execute(instruction_decoded);
    }

    #[inline(always)]
    fn fetch(self: &mut Self) -> u16 {
        if let (Some(high), Some(low)) = (
            self.ram.get(self.program_counter),
            self.ram.get(self.program_counter + 1),
        ) {
            self.program_counter += 2;
            ((high as u16) << 8) | (low as u16)
        } else {
            println!("PC out of bounds at 0x{:03X}.", self.program_counter);
            exit(1);
        }
    }

    fn decode(self: &Self, instruction: u16) -> Opcode {
        match (instruction & 0xF000) >> 12 {
            0x0 => match instruction {
                0x00E0 => Opcode::ClearScreen,
                0x00EE => Opcode::Return,
                _ => {
                    println!(
                        "Unknown instruction 0x{:04X} at PC 0x{:03X}.",
                        instruction,
                        self.program_counter - 2
                    );
                    exit(1);
                }
            },
            0x1 => Opcode::Jump(instruction & 0x0FFF),
            0x2 => Opcode::Call(instruction & 0x0FFF),
            0x3 => Opcode::SkipEq(
                ((instruction & 0x0F00) >> 8) as u8,
                (instruction & 0x00FF) as u8,
            ),
            0x4 => Opcode::SkipNeq(
                ((instruction & 0x0F00) >> 8) as u8,
                (instruction & 0x00FF) as u8,
            ),
            0x5 => {
                if instruction & 0x000F != 0 {
                    println!(
                        "Wrong instruction format - The fourth half byte of an instruction that starts with opcode 0x5 must be 0x0."
                    );
                    exit(1);
                };

                Opcode::SkipEqReg(
                    ((instruction & 0x0F00) >> 8) as u8,
                    ((instruction & 0x00F0) >> 4) as u8,
                )
            }
            0x6 => Opcode::LoadReg(
                ((instruction & 0x0F00) >> 8) as u8,
                (instruction & 0x00FF) as u8,
            ),
            0x7 => Opcode::AddImm(
                ((instruction & 0x0F00) >> 8) as u8,
                (instruction & 0x00FF) as u8,
            ),
            0x8 => match instruction & 0x000F {
                0x0 => Opcode::LoadRegReg(
                    ((instruction & 0x0F00) >> 8) as u8,
                    ((instruction & 0x00F0) >> 4) as u8,
                ),
                0x1 => Opcode::Or(
                    ((instruction & 0x0F00) >> 8) as u8,
                    ((instruction & 0x00F0) >> 4) as u8,
                ),
                0x2 => Opcode::And(
                    ((instruction & 0x0F00) >> 8) as u8,
                    ((instruction & 0x00F0) >> 4) as u8,
                ),
                0x3 => Opcode::Xor(
                    ((instruction & 0x0F00) >> 8) as u8,
                    ((instruction & 0x00F0) >> 4) as u8,
                ),
                0x4 => Opcode::AddReg(
                    ((instruction & 0x0F00) >> 8) as u8,
                    ((instruction & 0x00F0) >> 4) as u8,
                ),
                0x5 => Opcode::Sub(
                    ((instruction & 0x0F00) >> 8) as u8,
                    ((instruction & 0x00F0) >> 4) as u8,
                ),
                0x6 => Opcode::Shr(
                    ((instruction & 0x0F00) >> 8) as u8,
                    ((instruction & 0x00F0) >> 4) as u8,
                ),
                0x7 => Opcode::Subn(
                    ((instruction & 0x0F00) >> 8) as u8,
                    ((instruction & 0x00F0) >> 4) as u8,
                ),
                0xE => Opcode::Shl(
                    ((instruction & 0x0F00) >> 8) as u8,
                    ((instruction & 0x00F0) >> 4) as u8,
                ),
                _ => {
                    println!(
                        "Unknown instruction 0x{:04X} at PC 0x{:03X}.",
                        instruction,
                        self.program_counter - 2
                    );
                    exit(1);
                }
            },
            0x9 => {
                if instruction & 0x000F != 0 {
                    println!(
                        "Wrong instruction format - The fourth half byte of an instruction that starts with opcode 0x9 must be 0x0."
                    );
                    exit(1);
                };

                Opcode::SkipNeqReg(
                    ((instruction & 0x0F00) >> 8) as u8,
                    ((instruction & 0x00F0) >> 4) as u8,
                )
            }
            0xA => Opcode::LoadI(instruction & 0x0FFF),
            0xB => Opcode::JumpOffset(instruction & 0x0FFF),
            0xC => Opcode::Rand(
                ((instruction & 0x0F00) >> 8) as u8,
                (instruction & 0x00FF) as u8,
            ),
            0xD => Opcode::Draw(
                ((instruction & 0x0F00) >> 8) as u8,
                ((instruction & 0x00F0) >> 4) as u8,
                (instruction & 0x000F) as u8,
            ),
            0xE => match instruction & 0x00FF {
                0x9E => Opcode::SkipKey(((instruction & 0x0F00) >> 8) as u8),
                0xA1 => Opcode::SkipNotKey(((instruction & 0x0F00) >> 8) as u8),
                _ => {
                    println!(
                        "Unknown instruction 0x{:04X} at PC 0x{:03X}.",
                        instruction,
                        self.program_counter - 2
                    );
                    exit(1);
                }
            },
            0xF => match instruction & 0x00FF {
                0x07 => Opcode::GetDelay(((instruction & 0x0F00) >> 8) as u8),
                0x0A => Opcode::WaitKey(((instruction & 0x0F00) >> 8) as u8),
                0x15 => Opcode::SetDelay(((instruction & 0x0F00) >> 8) as u8),
                0x18 => Opcode::SetSound(((instruction & 0x0F00) >> 8) as u8),
                0x1E => Opcode::AddI(((instruction & 0x0F00) >> 8) as u8),
                0x29 => Opcode::LoadSprite(((instruction & 0x0F00) >> 8) as u8),
                0x33 => Opcode::Bcd(((instruction & 0x0F00) >> 8) as u8),
                0x55 => Opcode::StoreRegs(((instruction & 0x0F00) >> 8) as u8),
                0x65 => Opcode::LoadRegs(((instruction & 0x0F00) >> 8) as u8),
                _ => {
                    println!(
                        "Unknown instruction 0x{:04X} at PC 0x{:03X}.",
                        instruction,
                        self.program_counter - 2
                    );
                    exit(1);
                }
            },
            _ => unreachable!(),
        }
    }

    fn execute(self: &mut Self, instruction_decoded: Opcode) -> () {
        match instruction_decoded {
            Opcode::ClearScreen => {
                self.screen.clear();
                if !self.no_vertical_synchronization {
                    self.screen.synchronize();
                };
            }
            Opcode::Return => {
                self.stack_pointer -= 1;
                self.program_counter =
                    self.stack.get(self.stack_pointer).copied().unwrap() as usize;
                self.stack[self.stack_pointer] = 0;
            }
            Opcode::Jump(address) => self.program_counter = address as usize,
            Opcode::Call(address) => {
                if self.stack_pointer >= 16 {
                    println!(
                        "Call stack overflows while attempting to push return address 0x{:03X}.\nCall Stack (with Stack Pointer at 0xF, Old -> New Order):\n{}",
                        self.program_counter,
                        self.stack
                            .iter()
                            .map(|&x| format!("0x{:03X}", x))
                            .collect::<Vec<String>>()
                            .join(" ")
                    );
                    exit(1);
                };
                self.stack[self.stack_pointer] = self.program_counter as u16;
                self.stack_pointer += 1;
                self.program_counter = address as usize;
            }
            Opcode::SkipEq(x, n) => {
                if self.registers[x as usize] == n {
                    self.program_counter += 2;
                };
            }
            Opcode::SkipNeq(x, n) => {
                if self.registers[x as usize] != n {
                    self.program_counter += 2;
                };
            }
            Opcode::SkipEqReg(x, y) => {
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.program_counter += 2;
                };
            }
            Opcode::LoadReg(x, n) => self.registers[x as usize] = n,
            Opcode::AddImm(x, n) => {
                self.registers[x as usize] = self.registers[x as usize].wrapping_add(n);
            }
            Opcode::LoadRegReg(x, y) => self.registers[x as usize] = self.registers[y as usize],
            Opcode::Or(x, y) => {
                self.registers[x as usize] |= self.registers[y as usize];
                self.registers[0xF] = 0;
            }
            Opcode::And(x, y) => {
                self.registers[x as usize] &= self.registers[y as usize];
                self.registers[0xF] = 0;
            }
            Opcode::Xor(x, y) => {
                self.registers[x as usize] ^= self.registers[y as usize];
                self.registers[0xF] = 0;
            }
            Opcode::AddReg(x, y) => {
                let (result, overflowed) =
                    self.registers[x as usize].overflowing_add(self.registers[y as usize]);
                self.registers[x as usize] = result;
                self.registers[0xF] = overflowed as u8;
            }
            Opcode::Sub(x, y) => {
                let (result, borrowed) =
                    self.registers[x as usize].overflowing_sub(self.registers[y as usize]);
                self.registers[x as usize] = result;
                self.registers[0xF] = !borrowed as u8;
            }
            Opcode::Shr(x, y) => {
                let source: u8 = self.registers[y as usize];
                self.registers[x as usize] = self.registers[y as usize] >> 1;
                self.registers[0xF] = source & 0b0000_0001;
            }
            Opcode::Subn(x, y) => {
                let (result, borrowed) =
                    self.registers[y as usize].overflowing_sub(self.registers[x as usize]);
                self.registers[x as usize] = result;
                self.registers[0xF] = !borrowed as u8;
            }
            Opcode::Shl(x, y) => {
                let source: u8 = self.registers[y as usize];
                self.registers[x as usize] = self.registers[y as usize] << 1;
                self.registers[0xF] = (source >> 7) & 0b0000_0001;
            }
            Opcode::SkipNeqReg(x, y) => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.program_counter += 2;
                };
            }
            Opcode::LoadI(n) => self.index_register = n,
            Opcode::JumpOffset(offset) => {
                self.program_counter = (self.registers[0] as u16 + offset) as usize;
            }
            Opcode::Rand(x, n) => {
                self.registers[x as usize] = self.random_number_generator.next().unwrap() & n;
            }
            Opcode::Draw(x, y, n) => {
                let x_position: usize = self.registers[x as usize] as usize;
                let y_position: usize = self.registers[y as usize] as usize;
                let start_address: usize = self.index_register as usize;

                let mut sprite: Vec<u8> = Vec::with_capacity(n as usize);
                for i in 0..n {
                    match self.ram.get(start_address + i as usize) {
                        Some(byte) => sprite.push(byte),
                        None => {
                            println!(
                                "Sprite data out of RAM bounds at I {:#X}+{}",
                                self.index_register, i
                            );
                            exit(1);
                        }
                    }
                }

                self.registers[0xF] =
                    self.screen.draw_sprite(x_position, y_position, &sprite) as u8;

                if !self.no_vertical_synchronization {
                    self.screen.synchronize();
                };
            }
            Opcode::SkipKey(x) => {
                if self.current_keys[self.registers[x as usize] as usize] {
                    self.program_counter += 2;
                };
            }
            Opcode::SkipNotKey(x) => {
                if !self.current_keys[self.registers[x as usize] as usize] {
                    self.program_counter += 2;
                };
            }
            Opcode::GetDelay(x) => self.registers[x as usize] = self.delay_timer,
            Opcode::WaitKey(x) => {
                if !self.wait_key_released {
                    let mut is_pressed: bool = false;
                    for (i, key) in self.current_keys.iter().copied().enumerate() {
                        if key {
                            self.registers[x as usize] = i as u8;
                            is_pressed = true;
                            break;
                        };
                    }
                    if is_pressed {
                        self.wait_key_released = true;
                    };
                    self.program_counter -= 2;
                } else if !self.current_keys[self.registers[x as usize] as usize] {
                    self.wait_key_released = false;
                } else {
                    self.program_counter -= 2;
                };
            }
            Opcode::SetDelay(x) => self.delay_timer = self.registers[x as usize],
            Opcode::SetSound(x) => self.sound_timer = self.registers[x as usize],
            Opcode::AddI(x) => {
                self.index_register = self
                    .index_register
                    .wrapping_add(self.registers[x as usize] as u16);
            }
            Opcode::LoadSprite(x) => {
                self.index_register = 0x050 + (self.registers[x as usize] as u16) * 5;
            }
            Opcode::Bcd(x) => {
                let value: u8 = self.registers[x as usize];
                let hundreds: u8 = value / 100;
                let tens: u8 = (value / 10) % 10;
                let ones: u8 = value % 10;

                let _: Result<(), ()> = self.ram.set(self.index_register as usize, hundreds);
                let _: Result<(), ()> = self.ram.set(self.index_register as usize + 1, tens);
                let _: Result<(), ()> = self.ram.set(self.index_register as usize + 2, ones);
            }
            Opcode::StoreRegs(x) => {
                for i in 0..=x {
                    let _: Result<(), ()> = self.ram.set(
                        (self.index_register + i as u16) as usize,
                        self.registers[i as usize],
                    );
                }
                self.index_register += x as u16 + 1;
            }
            Opcode::LoadRegs(x) => {
                for i in 0..=x {
                    match self.ram.get((self.index_register + i as u16) as usize) {
                        Some(n) => self.registers[i as usize] = n,
                        None => {
                            println!("Unknown error occurred while accessing the RAM.");
                            exit(1);
                        }
                    };
                }
                self.index_register += x as u16 + 1;
            }
        };
    }
}
