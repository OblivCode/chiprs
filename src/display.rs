
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode};
use sdl2::rect::{Rect};
use sdl2::render::Canvas;
use std::sync::{Mutex, Arc, MutexGuard};
use std::thread;
use std::time::Duration;
use sdl2::video::Window;




pub fn start_display(scale_factor: u32,  delay: f32, key_mapping: [Keycode; 16], vm_lock: Arc<Mutex<[[u8; 64]; 32]>>, kp_lock: Arc<Mutex<[u8; 16]>>) {
        thread::spawn(move || {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("Chip8", 64*scale_factor, 32*scale_factor)
        .position_centered().opengl().build().unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        let mut event_pump = sdl_context.event_pump().unwrap();
        canvas.present();
            loop {
                thread::sleep(Duration::from_secs_f32(delay));
                update(&mut canvas, scale_factor, &vm_lock);
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                            panic!("Window quit")
                        }
                        Event::KeyDown { keycode, .. } => {
                            let mut keypad = kp_lock.lock().unwrap();
                            let key = keycode.unwrap();

                            let index = key_mapping.iter().position(|x| x.to_owned() == key);
                            if index.is_some(){
                                let idx = index.unwrap();
                                keypad[idx] = 1;
                            }
                        }
                        Event::KeyUp { keycode, .. } => {
                            let mut keypad = kp_lock.lock().unwrap();
                            let key = keycode.unwrap();

                            let index = key_mapping.iter().position(|x| x.to_owned() == key);
                            if index.is_some(){
                                let idx = index.unwrap();
                                keypad[idx] = 0;
                            }
                        }
                        _ => {}
                    }
                }
            }
        });

}

fn update(canvas: &mut Canvas<Window>, scale_factor: u32, vm_lock: &Arc<Mutex<[[u8; 64]; 32]>>) {
    let memory: MutexGuard<[[u8; 64]; 32]> = vm_lock.lock().unwrap();
    let pixel_on = Color::RGB(255,255,255);
    let pixel_off = Color::RGB(0,0,0);
    
    let mut display_pixel_y = 0;
    let mut display_pixel_x = 0;

    for y in 0..memory.len() {
        for x in  0..memory[0].len() {
            let pixel = memory[y][x];
            if pixel == 1 {
                canvas.set_draw_color(pixel_on);
                //println!("{} x {}", y,x);
            } else {
                canvas.set_draw_color(pixel_off);
            }

            canvas.fill_rect(Rect::new(display_pixel_x, display_pixel_y,  scale_factor , scale_factor)).expect("Could not draw pixel rect"); 
            display_pixel_x += scale_factor as i32;
        }
        display_pixel_x = 0;
        display_pixel_y += scale_factor as i32;
    }

    canvas.present();
}

