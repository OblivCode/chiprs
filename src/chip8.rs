use std::sync::{Arc, Mutex};

use rand::Rng;


fn int_to_hex<U: Into<usize>>(int: U) -> String {
    format!("{:#x}", int.into())
}
const FONTSET_START_ADDRESS: usize = 0x50;

pub struct Processor {   
    //registers
    registers: [u8; 16],
    index_register: u16,
    program_counter: u16,
    stack_pointer: u16,
    //memory
    vmemory_lock: Arc<Mutex<[[u8; 64]; 32]>>,
    memory: [u8; 4096],
    stack: [u16; 16],
    keypad_lock: Arc<Mutex<[u8; 16]>>,
    //timers
    sound_timer_lock: Arc<Mutex<u8>>,
    delay_timer_lock: Arc<Mutex<u8>>,
    //op
    opcode: u16,
    rom_start_address: usize,
}

impl Processor {
    pub fn new(fontset: [u8; 80]) -> Processor {
        //init chip8 processor
        let mut processor = Processor { registers: [0x0; 16], index_register: 0x0, program_counter: 0x200, 
            stack_pointer: 0, vmemory_lock: Arc::new(Mutex::new([[0; 64]; 32])), memory: [0x0; 4096], stack: [0x0; 16], 
            keypad_lock: Arc::new(Mutex::new([0x0; 16])), sound_timer_lock: Arc::new(Mutex::new(0)), delay_timer_lock:Arc::new(Mutex::new(0)), opcode: 0x0, 
            rom_start_address: 0x200 };
        //load fontset
        for idx in 0..fontset.len() {
            processor.memory[FONTSET_START_ADDRESS+idx] = fontset[idx];
        }

        return processor;
    }
    pub fn get_vmemory(&self) -> Arc<Mutex<[[u8; 64]; 32]>> {
        let clone: Arc<Mutex<[[u8; 64]; 32]>> = Arc::clone(&self.vmemory_lock);
        return clone;
    } 
    pub fn get_keypad(&self) -> Arc<Mutex<[u8; 16]>> {
        let clone = Arc::clone(&self.keypad_lock);
        return clone;
    }
    pub fn get_timers(&self) ->  (Arc<Mutex<u8>>,Arc<Mutex<u8>>) {
        let sclone = Arc::clone(&self.sound_timer_lock);
        let dclone = Arc::clone(&self.delay_timer_lock);
        return (dclone, sclone)
    }
    pub fn load_rom(&mut self, buffer: &Vec<u8>) {
        for count in 0..buffer.len() {
            self.memory[self.rom_start_address + count] = buffer[count];
        }
        println!("Loaded rom! {} bytes.", buffer.len())
    }

    pub fn cycle(&mut self) {
        let pc = self.program_counter as usize;
        self.opcode = ((self.memory[pc] as u16) << 8) | (self.memory[pc + 1] as u16);
        self.program_counter += 2;
        //println!("Fetched opcode: {}", int_to_hex(self.opcode));
        self.process_opcode();

        
    }

    fn process_opcode(&mut self) {
        let nibbles: [u8; 5] = [
            0 as u8, ((self.opcode & 0xF000) >> 12) as u8, ((self.opcode & 0x0F00) >> 8) as u8, 
            ((self.opcode & 0x00F0) >> 4) as u8, (self.opcode & 0xF) as u8];
        let rx = self.registers[nibbles[2] as usize];
        let ry = self.registers[nibbles[3] as usize];
        let x = nibbles[2];
        //let y = nibbles[3];
        let kk = (nibbles[3] << 4) + nibbles[4];
        let nnn = self.opcode & 0x0FFF;

        match nibbles[1] {
            0x0 => {
                match self.opcode & 0x00FF {
                    0xE0 => {
                        let mut vmemory = self.vmemory_lock.lock().unwrap();
                        *vmemory = [[0x0; 64];32]
                    }
                    0xEE => {
                        self.stack_pointer -= 1;
                        self.program_counter = self.stack[self.stack_pointer as usize];
                    }
                    _ => println!("Unhandled 0x0 opcode: {}", int_to_hex(self.opcode))
                } 
            },
            0x1 => { //#JP Jump to location 1[nnn]
                self.program_counter = nnn;
            },
            0x2 => { //#CALL call subroutine at 2[nnn]
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.stack_pointer += 1;
                self.program_counter = nnn;
            }
            0x3 => { //3xkk if register value at x equals kk then skip
                if rx == kk {
                    self.program_counter += 2;
                }
            }
            0x4 => {//4xkk opposite of SE
                if rx != kk {
                    self.program_counter += 2;
                }
            }
            0x5 => { //5xy0 if register value at x equals register value at y then skip
                if rx == ry {
                    self.program_counter += 2;
                }
            }
            0x6 => { //LD 6xkk set register value at x to kk
                self.registers[x as usize] = kk;
            }
            0x7 => { //0x7xkk add kk to register value at x
                let sum = rx as u16 + kk as u16;
                self.registers[x as usize] = sum as u8;
            }
            0x8 => {
                match nibbles[4] {
                    0x0 => {
                        //set value at register x to register value y
                        self.registers[x as usize] = ry;
                    }
                    0x1 => {
                        //set value at register x  = register x OR register y
                        self.registers[x as usize] |= ry;
                    }
                    0x2 => {
                        //register x = register x AND register y
                        self.registers[x as usize] &= ry;
                    }
                    0x3 => {
                        //register x = register x XOR register y
                        self.registers[x as usize] ^= ry;
                    }
                    0x4 => {
                        //add register x and y then set registerF to 1 if sum over 255 (8 bits)
                        let sum = rx as u16 + ry as u16;
                        if sum > 255 {
                            self.registers[0xF] = 1
                        } else {
                            self.registers[0xF] = 0
                        }
                        self.registers[x as usize] = (sum & 0xFF) as u8;
                    }
                    0x5 => {
                        //sub register y from register x then set register 0xF to 1 if no borrow / x > y
                        let no_borrow = rx > ry;
                        if no_borrow {
                            
                            self.registers[0xF] = 1;
                        }
                        else {
                            self.registers[0xF] = 0
                        }
                        self.registers[x as usize] = rx.wrapping_sub(ry);
                    }
                    0x6 => {
                        // if least-signficiant bit of Rx value is 1 then set 0xF to 1 else 0 then Rx value divide by 2
                        let lsb = rx & 0x1;
                        self.registers[0xF] = lsb;
                        self.registers[x as usize] >>= 1; //divide by 2
                    }
                    0x7 => {
                        //if Ry > Rx then set 0xF to 1 else 0 then sub Rx from Ry and store result in Rx
                        if ry > rx {
                            self.registers[0xF] = 1
                        } else {
                            self.registers[0xF] = 0
                        }
                        self.registers[x as usize] = rx.wrapping_sub(ry);
                    }
                    0xE => {
                        //if most-significant bit of Rx is 1 then set 0xF to 1 else 0 then Rx multiply by 2
                        let msb = (self.registers[x as usize] & 0x80) >> 7;
                        self.registers[0xF] = msb;
                        self.registers[x as usize] <<=1; //multiply by 2
                    }
                    _ => println!("Unhandled 0x8 opcode: {}", self.opcode)
                }
            }
            0x9 => { //9xy0 skip next instruction if Rx != Ry
                if rx != ry {
                    self.program_counter += 2;
                }
            },
            0xA => {//A[nnn] set Rindex = nnn
                self.index_register = nnn;
            }
            0xB => { //Jump to location nnn + R0
                self.program_counter = nnn + self.registers[0] as u16;
            }
            0xC => {
                //Cxkk Rx = random byte AND kk
                let mut rng = rand::thread_rng();

                let random_byte: u8 = rng.gen();
                self.registers[x as usize] = random_byte & kk;
            }
            0xD => {//#DRW 0xDxyn Display [n] byte sprite starting at memory location   I at (Rx, Ry), set 0xF = collison
                //#sprite always eight pixels wide so [n] = height
                self.registers[0xF] = 0;
                let mut vm = self.vmemory_lock.lock().unwrap();
                 //       *vmemory = [[0x0; 64];32]
                let row_count = vm.len();
                let col_count = vm[0].len();
                let n = nibbles[4] as usize;
                let posx = rx as usize % col_count;
                let posy = ry as usize % row_count;

                for row_num in 0..n {
                    let sprite_byte = self.memory[self.index_register as usize + row_num];

                    for col_num in  0..8 {
                        let sprite_pixel = sprite_byte & (0x80 >> col_num);

                        let y_idx = posy+row_num;
                        let x_idx = posx+col_num;
                        
                        if y_idx > vm.len()-1 || x_idx > vm[0].len()-1 {
                            continue
                        }

                        let mut screen_pixel = vm[y_idx][x_idx];

                        if sprite_pixel != 0 {
                            if screen_pixel == 1 {
                                self.registers[0xF] = 1
                            }
                            screen_pixel ^= 0x1;
                        }
                        vm[y_idx][x_idx] = screen_pixel;
                    }
                }
            }
            0xE => {
                let second_byte = (nibbles[3] << 4) + nibbles[4];
                let keypad = self.keypad_lock.lock().unwrap();
                match second_byte {
                    //Ex9E skip next instruction if key with the value of Rx is pressed
                    0x9E => {
                        if keypad[rx as usize] == 1 {
                            self.program_counter += 2;
                        }
                    }
                    //0xExA1 skip next instruction if key with the value of Rx is not pressed
                    0xA1 => {
                        if keypad[rx as usize] == 0{
                            self.program_counter += 2;
                        }
                    }
                    _ => println!("Unhandled 0xE opcode: {}", int_to_hex(second_byte))
                }
            }
            0xF => {
                let second_byte = (nibbles[3] << 4) + nibbles[4];

                match second_byte {
                    0x07 => {
                        //rx = delay timer value
                        let delay_timer = self.delay_timer_lock.lock().unwrap();
                        self.registers[x as usize] = *delay_timer;
                    }
                    0x0A => { 
                        //Fx0A Wait for key press then store key value in Rx
                        let mut pressed = false;
                        let keypad = self.keypad_lock.lock().unwrap();
                        for idx in 0..keypad.len() {
                            if keypad[idx] == 1 {
                                self.registers[x as usize] = idx as u8;
                                pressed = true;
                                break;
                            }
                        }
                        if !pressed {
                            self.program_counter -= 2;
                        }
                    }
                    0x15 => {
                        //Set delay timer = rx
                        let mut delay_timer = self.delay_timer_lock.lock().unwrap();
                        *delay_timer = rx;
                    }
                    0x18 => {
                        //sound timer = rx
                        let mut sound_timer = self.sound_timer_lock.lock().unwrap();
                        *sound_timer = rx;
                    }
                    0x1E => { //Fx1E index = index + Rx
                        self.index_register += rx as u16;
                    }
                    0x29 => {
                        //index = location of sprite for digit Rx
                        //font is 5 bytes each and start a fontset start addr
                        //Rx = position of font character
                        self.index_register = (FONTSET_START_ADDRESS as u8 + (5*rx)) as u16;
                    }
                    0x33 => {
                        /*Store BCD representation of Rx in memory locations I, I+1, and I+2.
                        The interpreter takes the decimal value of Rx, and places the hundreds 
                        digit in memory at location in I, the tens digit at location I+1, and 
                        the ones digit at location I+2. */
                        //value = Rx
                        let mut value = rx;
                        let index = self.index_register as usize;
                        self.memory[index + 2] = value % 10;
                        value /= 10;
                        self.memory[index + 1] =value % 10;
                        value /= 10;
                        self.memory[index] = value % 10;
                    }
                    0x55 => {
                        //Store registers R0 through R[x] in memory starting at location Index.
                        for idx in 0..(x+1) as usize {
                            self.memory[self.index_register as usize + idx] = self.registers[idx];
                        }
                    }
                    0x65 => {
                        //Read registers R0 through R[x] from memory starting at location Index.
                        for idx in 0..(x+1) as usize {
                            self.registers[idx] = self.memory[idx+self.index_register as usize];
                        }
                    }
                    _ => {
                        println!("Unhandled 0xF opcode: {}", int_to_hex(self.opcode))
                    }
                }
            }
            _ => println!("Unhandled opcode: {}", int_to_hex(self.opcode))
        }
    }

}
