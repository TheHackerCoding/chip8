use self::utils::random;
use smallvec::{smallvec, SmallVec};
use std::{error::Error, fs, process::exit};
#[path = "./utils.rs"]
mod utils;

const CHIP8_FONTSET: SmallVec<[u8; 80]> = SmallVec::from_const([
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
    0x90, 0x90, 0xF0, 0x10, 0x10, //4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
    0xF0, 0x10, 0x20, 0x40, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80, //F
]);
#[derive(Debug, Clone)]
pub struct Chip8 {
    opcode: u16,
    // limit is 4096
    memory: SmallVec<[u8; 4096]>,
    // limit is 16
    V: SmallVec<[u8; 16]>,
    I: u8,
    pc: u16,
    // limit is 2048
    pub gfx: SmallVec<[u8; 2048]>,
    delay_timer: u8,
    sound_timer: u8,
    // limit is 16
    stack: SmallVec<[u8; 16]>,
    sp: u8,
    // limit is 16
    key: SmallVec<[u8; 16]>,
    pub drawFlag: bool,
}

impl Chip8 {
    pub fn init() -> Chip8 {
        let mut memory: SmallVec<[u8; 4096]> = SmallVec::from_vec(CHIP8_FONTSET.to_vec());
        // smh i could probably do a better job at this
        unsafe { memory.set_len(4096) }
        Chip8 {
            opcode: 0,
            memory,
            V: smallvec![0; 16],
            I: 0,
            pc: 0x200,
            gfx: smallvec![0; 2048],
            delay_timer: 0,
            sound_timer: 0,
            stack: smallvec![0; 16],
            sp: 0,
            key: smallvec![0; 16],
            drawFlag: true,
        }
    }

    pub fn emulate_cycle(&mut self) {
        let mut memory = &mut self.memory;
        let mut pc = self.pc;
        let mut opcode = self.opcode;
        let mut gfx = &mut self.gfx;
        let mut drawFlag = self.drawFlag;
        let mut sp = self.sp;
        let mut stack = &mut self.stack;
        let mut sound_timer = self.sound_timer;
        let mut delay_timer = self.delay_timer;
        let mut V = &mut self.V;
        let mut I = self.I;
        let mut key = &self.key;

        //let Chip8 {
        //    opcode,
        //    memory,
        //    V,
        //    I,
        //    pc,
        //    gfx,
        //    delay_timer,
        //    sound_timer,
        //    stack,
        //    sp,
        //    key,
        //    drawFlag,
        //} = self;

        opcode = ((((memory[pc as usize]) as u16) << 8) as u8 | memory[(pc + 1) as usize]) as u16;

        match opcode & 0xF000 {
            0x0000 => match opcode & 0x000F {
                0x0000 => {
                    // 0x00E0: Clears the screen
                    gfx = &mut smallvec![0; 2048];
                    drawFlag = true;
                    pc += 2;
                }

                0x000E => {
                    // 0x00EE: Returns from subroutine
                    sp -= 1;
                    pc = stack[sp as usize].into();
                    pc += 2;
                }
                _ => println!("Unknown opcode [0x0000]: 0x{}", opcode),
            },

            // 0x1NNN: Jumps to address NNN
            0x1000 => pc = opcode & 0x0FFF,

            0x2000 => {
                // 0x2NNN: Calls subroutine at NNN.
                stack[sp as usize] = pc as u8;
                sp += 1;
                pc = opcode & 0x0FFF;
            }

            0x3000 => {
                // 0x3XNN: Skips the next instruction if VX equals NN
                if V[((opcode & 0x0F00) >> 8) as usize] as u16 == (opcode & 0x00FF) {
                    pc += 4;
                } else {
                    pc += 2;
                }
            }

            0x4000 => {
                // 0x4XNN: Skips the next instruction if VX doesn't equal NN
                if V[((opcode & 0x0F00) >> 8) as usize] as u16 != (opcode & 0x00FF) {
                    pc += 4;
                } else {
                    pc += 2;
                }
            }

            0x5000 => {
                // 0x5XY0: Skips the next instruction if VX equals VY.
                if V[((opcode & 0x0F00) >> 8) as usize] == V[((opcode & 0x00F0) >> 4) as usize] {
                    pc += 4;
                } else {
                    pc += 2;
                }
            }

            0x6000 => {
                // 0x6XNN: Sets VX to NN.
                V[((opcode & 0x0F00) >> 8) as usize] = (opcode & 0x00FF) as u8;
                pc += 2;
            }

            0x7000 => {
                // 0x7XNN: Adds NN to VX.
                V[((opcode & 0x0F00) >> 8) as usize] = (opcode & 0x00FF) as u8;
                pc += 2;
            }

            0x8000 => match opcode & 0x000F {
                0x0000 => {
                    // 0x8XY0: Sets VX to the value of VY
                    V[((opcode & 0x0F00) >> 8) as usize] = V[((opcode & 0x00F0) >> 4) as usize];
                    pc += 2;
                }

                0x0001 => {
                    // 0x8XY1: Sets VX to "VX OR VY"
                    V[((opcode & 0x0F00) >> 8) as usize] |= V[((opcode & 0x00F0) >> 4) as usize];
                    pc += 2;
                }

                0x0002 => {
                    // 0x8XY2: Sets VX to "VX AND VY"
                    V[((opcode & 0x0F00) >> 8) as usize] &= V[((opcode & 0x00F0) >> 4) as usize];
                    pc += 2;
                }

                0x0003 => {
                    // 0x8XY3: Sets VX to "VX XOR VY"
                    V[((opcode & 0x0F00) >> 8) as usize] ^= V[((opcode & 0x00F0) >> 4) as usize];
                    pc += 2;
                }

                0x0004 => {
                    // 0x8XY4: Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't
                    if (V[((opcode & 0x00F0) >> 4) as usize]
                        > (0xFF - V[((opcode & 0x0F00) >> 8) as usize]))
                    {
                        V[0xF] = 1; // carry
                    } else {
                        V[0xF] = 0;
                    }
                    V[((opcode & 0x0F00) >> 8) as usize] += V[((opcode & 0x00F0) >> 4) as usize];
                    pc += 2;
                }

                0x0005 => {
                    // 0x8XY5: VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't
                    if V[((opcode & 0x00F0) >> 4) as usize] > V[((opcode & 0x0F00) >> 8) as usize] {
                        V[0xF] = 0; // there is a borrow
                    } else {
                        V[0xF] = 1;
                    }
                }

                0x0006 => {
                    // 0x8XY6: Shifts VX right by one. VF is set to the value of the least significant bit of VX before the shift
                    V[0xF] = V[((opcode & 0x0F00) >> 8) as usize] & 0x1;
                    V[((opcode & 0x0F00) >> 8) as usize] >>= 1;
                    pc += 2;
                }

                0x0007 => {
                    // 0x8XY7: Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn't
                    if V[((opcode & 0x0F00) >> 8) as usize] > V[((opcode & 0x00F0) >> 4) as usize] {
                        // VY-VX
                        V[0xF] = 0; // there is a borrow
                    } else {
                        V[0xF] = 1;
                    }
                    V[((opcode & 0x0F00) >> 8) as usize] =
                        V[((opcode & 0x00F0) >> 4) as usize] - V[((opcode & 0x0F00) >> 8) as usize];
                    pc += 2;
                }

                0x000E => {
                    // 0x8XYE: Shifts VX left by one. VF is set to the value of the most significant bit of VX before the shift
                    V[0xF] = V[((opcode & 0x0F00) >> 8) as usize] >> 7;
                    V[((opcode & 0x0F00) >> 8) as usize] <<= 1;
                    pc += 2;
                }

                _ => println!("Unknown opcode [0x8000]: 0x{}", opcode),
            },

            Ox9000 => {
                // 0x9XY0: Skips the next instruction if VX doesn't equal VY
                if V[((opcode & 0x0F00) >> 8) as usize] != V[((opcode & 0x00F0) >> 4) as usize] {
                    pc += 4;
                } else {
                    pc += 2;
                }
            }

            0xA000 => {
                // ANNN: Sets I to the address NNN
                I = (opcode & 0x0FFF) as u8;
                pc += 2;
            }

            0xB000 => {
                // BNNN: Jumps to the address NNN plus V0
                pc = (opcode & 0x0FFF) + V[0] as u16;
            }

            0xC000 => {
                // CXNN: Sets VX to a random number and NN
                V[((opcode & 0x0F00) >> 8) as usize] = ((random(0xFF)) & (opcode & 0x00FF)) as u8;
                pc += 2;
            }

            0xD000 => {
                // DXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
                // Each row of 8 pixels is read as bit-coded starting from memory location I;
                // I value doesn't change after the execution of this instruction.
                // VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn,
                // and to 0 if that doesn't happen

                let mut x = V[((opcode & 0x0F00) >> 8) as usize];
                let mut y = V[((opcode & 0x00F0) >> 4) as usize];
                let mut height = opcode & 0x000F;
                let mut pixel: u8;

                V[0xF] = 0;
                let mut yline: u8 = 0;
                while yline < height as u8 {
                    yline += 1;
                    pixel = memory[(I + yline) as usize];
                    let mut xline: u8 = 0;
                    while xline < 8 {
                        xline += 1;
                        if (pixel & (0x80 >> xline)) != 0 {
                            if gfx[(x + xline + ((y + yline) * 64)) as usize] == 1 {
                                V[0xF] = 1;
                            }
                            gfx[(x + xline + ((y + yline) * 64)) as usize] ^= 1;
                        }
                    }
                }
                drawFlag = true;
                pc += 2;
            }

            0xE000 => match opcode & 0x00FF {
                0x009E => {
                    // EX9E: Skips the next instruction if the key stored in VX is pressed
                    if key[V[((opcode & 0x0F00) >> 8) as usize] as usize] != 0 {
                        pc += 4;
                    } else {
                        pc += 2;
                    }
                }

                0x00A1 => {
                    // EXA1: Skips the next instruction if the key stored in VX isn't pressed
                    if key[V[((opcode & 0x0F00) >> 8) as usize] as usize] == 0 {
                        pc += 4;
                    } else {
                        pc += 2;
                    }
                }

                _ => println!("Unknown opcode [0xE000]: 0x{}", opcode),
            },

            0xF000 => match opcode & 0x00FF {
                0x0007 => {
                    // FX07: Sets VX to the value of the delay timer
                    V[((opcode & 0x0F00) >> 8) as usize] = delay_timer;
                    pc += 2;
                }

                0x000A => {
                    // FX0A: A key press is awaited, and then stored in VX
                    let mut keyPress: bool = false;
                    for i in 0..16 {
                        if key[i] == 0 {
                            V[((opcode & 0x0F00) >> 8) as usize] = i as u8;
                            keyPress = true;
                        }
                    }
                    if !keyPress {
                        return;
                    }
                    pc += 2;
                }

                0x0015 => {
                    // FX15: Sets the delay timer to VX
                    delay_timer = V[((opcode & 0x0F00) >> 8) as usize];
                    pc += 2;
                }

                0x0018 => {
                    // FX18: Sets the sound timer to VX
                    sound_timer = V[((opcode & 0x0F00) >> 8) as usize];
                    pc += 2;
                }

                0x001E => {
                    // FX1E: Adds VX to I
                    if (I + V[((opcode & 0x0F00) >> 8) as usize]) as u16 > 0xFFF {
                        // VF is set to 1 when range overflow (I+VX>0xFFF), and 0 when there isn't.
                        V[0xF] = 1;
                    } else {
                        V[0xF] = 0;
                    }
                    I += V[((opcode & 0x0F00) >> 8) as usize];
                    pc += 2;
                }

                0x0029 => {
                    // FX29: Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font
                    I = V[((opcode & 0x0F00) >> 8) as usize] * 0x5;
                    pc += 2;
                }

                0x0033 => {
                    // FX33: Stores the Binary-coded decimal representation of VX at the addresses I, I plus 1, and I plus 2
                    memory[I as usize] = V[((opcode & 0x0F00) >> 8) as usize] / 100;
                    memory[(I + 1) as usize] = (V[((opcode & 0x0F00) >> 8) as usize] / 10) % 10;
                    memory[(I + 2) as usize] = (V[((opcode & 0x0F00) >> 8) as usize] % 100) % 10;
                    pc += 2;
                }

                0x0055 => {
                    // FX55: Stores V0 to VX in memory starting at address I
                    let mut i = 0;
                    while i <= (opcode & 0x0F00) >> 8 {
                        i += 1;
                        memory[(I + i as u8) as usize] = V[i as usize];
                    }
                    // On the original interpreter, when the operation is done, I = I + X + 1.
                    I += ((opcode & 0x0F00) >> 8) as u8 + 1;
                    pc += 2;
                }

                0x0065 => {
                    // FX65: Fills V0 to VX with values from memory starting at address I
                    let mut i = 0;
                    while i <= (opcode & 0x0F00) >> 8 {
                        i += 1;
                        V[i as usize] = memory[(I + i as u8) as usize];
                    }
                    // On the original interpreter, when the operation is done, I = I + X + 1.
                    I += ((opcode & 0x0F00) >> 8) as u8 + 1;
                    pc += 2;
                }
                _ => println!("Unknown opcode [0xF000]: 0x{}", opcode),
            },

            _ => println!("Unknown opcode: 0x{}", opcode),
        }

        if delay_timer > 0 {
            delay_timer -= 1;
        }

        if sound_timer > 0 {
            if sound_timer == 1 {
                println!("BEEP!");
            }
            sound_timer -= 1;
        }
    }

    pub fn debug_render(&self) {
        for y in 0..32 {
            for x in 0..64 {
                if (self.gfx[(y * 64) + x] == 0) {
                    print!("0")
                } else {
                    print!(" ")
                }
            }
            println!()
        }
        println!()
    }

    pub fn load_application(&mut self, filename: &str) {
        let contents = fs::read(filename).expect("Unable to read/get file");
        let c_length = contents.len();
        println!("Filesize: {}", c_length);
        if 4096 - 512 > c_length {
            for i in 0..c_length {
                //println!("{}", i);
                self.memory[i + 512] = contents[i];
            }
        } else {
            println!("Error: ROM too big for memory");
            exit(1);
        }
    }
}
