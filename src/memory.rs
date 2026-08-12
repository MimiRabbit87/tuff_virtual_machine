use std::{fs::File, io::Read, path::Path};

pub struct RAM<const C: usize> {
    content: [u8; C],
}

#[allow(unused)]
impl<const C: usize> RAM<C> {
    pub fn new() -> Self {
        RAM { content: [0; C] }
    }

    pub fn load_from_file(self: &mut Self, file_path: &Path, offset: usize) -> () {
        let mut file: File = File::open(file_path).expect("failed to open file");
        let mut file_content: Vec<u8> = Vec::new();
        file.read_to_end(&mut file_content)
            .expect("failed to read file");
        self.load_from_array(&file_content, offset);
    }

    pub fn load_from_array(self: &mut Self, content: &[u8], offset: usize) -> () {
        if content.len() > C {
            println!(
                "Overflowed while loading content, whose length is {}.",
                content.len()
            );
            std::process::exit(1);
        }
        for i in 0..content.len() {
            self.content[offset + i] = content[i];
        }
    }

    #[inline(always)]
    pub fn get(self: &Self, address: usize) -> Option<u8> {
        self.content.get(address).copied()
    }

    #[inline(always)]
    pub fn set(self: &mut Self, address: usize, value: u8) -> Result<(), ()> {
        if address < C {
            self.content[address] = value;
            Ok(())
        } else {
            Err(())
        }
    }
}
