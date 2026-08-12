use std::sync::{Arc, Condvar, Mutex, MutexGuard};

#[derive(Clone)]
pub struct Screen {
    buffer: [bool; 64 * 32],
    draw_flag: Arc<(Mutex<bool>, Condvar)>,
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            buffer: [false; 64 * 32],
            draw_flag: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    #[inline(always)]
    pub fn get_buffer(self: &Self) -> [bool; 64 * 32] {
        self.buffer
    }

    pub fn draw_sprite(self: &mut Self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let x_start: usize = x % 64;
        let y_start: usize = y % 32;

        let mut collision: bool = false;
        for (row, &byte) in sprite.iter().enumerate() {
            let y_position: usize = y_start + row;
            if y_position >= 32 {
                break;
            };
            for column in 0..8 {
                let x_position: usize = x_start + column;
                if x_position >= 64 {
                    break;
                };
                if (byte >> (7 - column)) & 1 == 1 {
                    if self.xor_pixel(x_position, y_position, true) {
                        collision = true;
                    };
                };
            }
        }
        collision
    }

    #[inline(always)]
    pub fn clear(self: &mut Self) -> () {
        self.buffer = [false; 64 * 32];
    }

    #[inline(always)]
    fn xor_pixel(self: &mut Self, x_position: usize, y_position: usize, set: bool) -> bool {
        let old: bool = self.buffer[y_position * 64 + x_position];
        self.buffer[y_position * 64 + x_position] ^= set;
        old && set
    }

    pub fn set_drew(self: &Self) -> () {
        let (lock, cvar) = &*self.draw_flag;
        let mut drew: MutexGuard<'_, bool> = lock.lock().unwrap();
        *drew = true;
        cvar.notify_all();
    }

    pub fn synchronize(self: &Self) -> () {
        let (lock, cvar) = &*self.draw_flag;
        let mut drew: MutexGuard<'_, bool> = lock.lock().unwrap();
        while !*drew {
            drew = cvar.wait(drew).unwrap();
        }
        *drew = false;
    }
}
