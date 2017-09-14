#[macro_use]
extern crate lazy_static;
extern crate sdl2;
extern crate time;

use sdl2::audio::{AudioSpecDesired, AudioCallback};
use std::io;
use std::io::{BufRead};
use std::str::FromStr;
use std::time::Duration;

mod synth;
mod music;

static SAMPLE_RATE: i32 = 44100;


struct Note {
    struck: u64, // ns
    frequency: f32,
    duration: u64, // ns
}

struct PolyPhone {
    notes: Vec<Note>,
    clock: u64, // ns
}

impl PolyPhone {
    fn new() -> PolyPhone {
        PolyPhone {
            clock: 0,
            notes: Vec::new()
        }
    }

    fn play(&mut self, frequency: f32, duration: u64) {
        self.notes.push(Note {
            struck: self.clock,
            frequency,
            duration,
        });
    }
}

impl AudioCallback for PolyPhone {

    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            self.clock += 1000000000 / SAMPLE_RATE as u64;
            *x = self.notes.iter().map(
                |note| {
                    let envelope = synth::adsr((self.clock - note.struck) as f32, note.duration as f32);
                    let wave = (std::f32::consts::PI * 2.0 * self.clock as f32 / 1000000000.0).sin();
                    envelope * wave * 8000.0
                }
            ).fold(0.0, |a, b| a + b);
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let scale = music::scale(music::IONIAN.to_vec(), -9 - 48);
    println!("{:?}", scale);


    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE),
        channels: Some(1),
        // mono  -
        samples: Some(4096),
        // default sample size 
        };

    let device = audio_subsystem.open_queue::<i16, _>(None, &desired_spec).unwrap();
    // let temp_wave = gen_wave(44100, 440.0);
    // device.queue(&temp_wave);


    device.resume();
    let mut then = time::precise_time_ns();
    let mut bufuntil = time::precise_time_ns();

    for line in io::BufReader::new(io::stdin()).lines() {
        let now = time::precise_time_ns();
        let dt = now - then;
        then = now;
        if bufuntil < now {
            bufuntil = now;
        } else {
            bufuntil = bufuntil - dt; //
        }
        let real_line = line.unwrap();
        let parts: Vec<&str> = real_line.split(' ').collect();
        let pkt_length = parts[parts.len()-1];
        let pkt_size = match f32::from_str(pkt_length) {
            Ok(p) => {
                if p == 0.0 {
                    continue;
                }
                p.log(4.0)
            },
            Err(e) => {
                continue;
            }
        };
        let octets: Vec<&str> = parts[2].split('.').collect();
        // let first = (
        //     usize::from_str_radix(octets[0], 10).unwrap() +
        //     usize::from_str_radix(octets[1], 10).unwrap() +
        //     usize::from_str_radix(octets[2], 10).unwrap() +
        //     usize::from_str_radix(octets[3], 10).unwrap()
        // ) / 64;
        let first = usize::from_str_radix(octets[3], 10).unwrap() / 16;
        let note = scale[first + 16].clone();
        let samples = (SAMPLE_RATE / (32 / pkt_size.floor() as i32)) as i32;
        let note_length_ns = ((samples as u64) / 44100) * 1000000000; 
        if note_length_ns > 50000000 {
            continue;
        }
        if bufuntil - now < 2 * 1000000000 {
            let wave = synth::gen_wave(samples, note, SAMPLE_RATE);
            bufuntil += note_length_ns;
            device.queue(&wave);
        }
    }
    // let wave = gen_wave(target_bytes, 440);
    // device.queue(&wave);
    // Start playback 

    // Play for 2 seconds 
    std::thread::sleep(Duration::from_millis(2000));

    // Device is automatically closed when dropped 
}