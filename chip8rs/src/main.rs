use std::{fs, thread, time::Duration, sync::{Mutex, Arc}, path::PathBuf};
use native_dialog::{FileDialog, MessageType, MessageDialog};
use sdl2::keyboard::Keycode;
use winconsole::console;
use crate::{display::start_display, chip8::Processor};

mod chip8;
mod display;
const FONTSET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];
const KEY_MAPPING: [Keycode; 16] = [
    Keycode::Num1, Keycode::Num2, Keycode::Num3, Keycode::Num4,
    Keycode::Q, Keycode::W, Keycode::E, Keycode::R,
    Keycode::A, Keycode::S, Keycode::D, Keycode::F,
    Keycode::Z, Keycode::X, Keycode::C, Keycode::V
];
fn main() {
    
    let (clock_speed, scale_factor, refresh_rate) = (1000, 10, 30);
    let (clock_delay, refresh_delay) = (1.0 / clock_speed as f32, 1.0 / refresh_rate as f32); //ms
    
    let filename: String = get_romfile().unwrap();
    
    let rom_data: Vec<u8> = fs::read(&filename)
        .expect("Could not read from selected filename");
    let mut chip: Processor = chip8::Processor::new(FONTSET);
    chip.load_rom(&rom_data);
    
    //get locks from chip8 and pass to display and 60hz timer
    let vm_lock: Arc<Mutex<[[u8; 64]; 32]>> = chip.get_vmemory();
    let kp_lock: Arc<Mutex<[u8; 16]>> = chip.get_keypad();
    let timer_locks: (Arc<Mutex<u8>>,Arc<Mutex<u8>>) = chip.get_timers();
    start_display(scale_factor, refresh_delay, KEY_MAPPING, vm_lock, kp_lock);
    start_timers(timer_locks);
    
    println!("Rom file path: {}", &filename);
    println!("Scale factor: {}x so resolution of {}x{}", &scale_factor, 64*scale_factor, 32*scale_factor);
    println!("Refresh rate: {}hz so delay time of {} seconds", &refresh_rate, &refresh_delay);
    println!("Processor running at {}hz so delay time of {} seconds", clock_speed, &clock_delay);

    loop {
        thread::sleep(Duration::from_secs_f32(clock_delay));
        chip.cycle();
    }
}

fn get_romfile() -> Option<String> {
    let path: Option<PathBuf> = FileDialog::new()
        .set_location("~/Desktop")
        .show_open_single_file()
        .unwrap();

    let path: PathBuf = match path {
        Some(path) => path,
        None => return None,
    };

    let yes: bool = MessageDialog::new()
        .set_type(MessageType::Info)
        .set_title("Do you want to play this file?")
        .set_text(&format!("{:#?}", path))
        .show_confirm()
        .unwrap();
    if yes {
        return Some(path.to_str().unwrap().to_string());
    } else {
        return get_romfile();
    }
}

fn start_timers(timers: (Arc<Mutex<u8>>,Arc<Mutex<u8>>)) {
    let delay_lock: Arc<Mutex<u8>> = timers.0;
    let sound_lock: Arc<Mutex<u8>> = timers.1;
    let delay: f32 = 1.0 / 60.0;

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs_f32(delay));
            let (mut delay_timer, mut sound_timer) = (delay_lock.lock().unwrap(), sound_lock.lock().unwrap());
            if *delay_timer > 0{
                *delay_timer -= 1
            }
            if *sound_timer > 0 {
                console::beep(800, 50);
                *sound_timer -= 1
            }
    
        }
    });
}