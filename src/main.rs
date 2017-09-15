#[macro_use]
extern crate lazy_static;
extern crate sdl2;
extern crate time;

use sdl2::audio::{AudioSpecDesired, AudioCallback};
use std::io;
use std::io::{BufRead};
use std::str::FromStr;
use std::time::Duration;
use std::sync::mpsc::{channel, Receiver};

mod synth;
mod music;

const SAMPLE_RATE: i32 = 44100;
const QUANTIZE_NS: u64 = (1000000000 / (SAMPLE_RATE as u64));

#[derive(Debug, PartialEq, Eq)]
struct Note {
    struck: u64, // ns
    frequency: u64, // nano-hz
    duration: u64, // ns
}

struct PolyPhone {
    notes: Vec<Note>,
    clock: u64, // ns
    rx: Receiver<(f32, u64, u64)>,
}

impl PolyPhone {
    fn new(rx: Receiver<(f32, u64, u64)>) -> PolyPhone {
        PolyPhone {
            clock: 0,
            notes: Vec::new(),
            rx
        }
    }
}

impl AudioCallback for PolyPhone {

    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for item in self.rx.try_iter() {
            match item {
                (frequency, duration, struck) => {
                    let n = Note {
                        struck,
                        frequency: (frequency * 1000000000.0) as u64,
                        duration,
                    };
                    if self.notes.len() < 24 {
                        self.notes.push(n);
                    }
                }
            }
        }
        for x in out.iter_mut() {
            self.clock += 1000000000 / (SAMPLE_RATE as u64);
            *x = self.notes.iter()
            .map(
                |note| {
                    let envelope = synth::adsr((self.clock as i64 - note.struck as i64) as f32, note.duration as f32);
                    let t = (self.clock as f32) / 1000000000.0; // seconds
                    let f = (note.frequency as f32) / 1000000000.0; // Hz
                    let wave = (f * std::f32::consts::PI * 2.0 * t).sin();
                    envelope * wave * 1.0
                }
            ).fold(0.0, |a, b| a + b) / (self.notes.len() as f32);
        }
        self.notes = self.notes.iter()
        .filter(
            |note| note.struck + note.duration > self.clock
        )
        .map(
            |note| Note {
                duration: note.duration,
                struck: note.struck,
                frequency: note.frequency,
            }
        ).collect();
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let scale = music::scale(music::AEOLIAN.to_vec(), -9 - 48);
    println!("{:?}", scale);


    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE),
        channels: Some(1),
        // mono  -
        samples: Some(2048),
        // default sample size 
        };

    // let device = audio_subsystem.open_queue::<i16, _>(None, &desired_spec).unwrap();
    let (tx, rx) = channel();
    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        PolyPhone::new(rx)
    }).unwrap();
    // let temp_wave = gen_wave(44100, 440.0);
    // device.queue(&temp_wave);


    device.resume();
    let start: u64 = time::precise_time_ns(); 

    for line in io::BufReader::new(io::stdin()).lines() {
        let clock = time::precise_time_ns() - start;
        let real_line = line.unwrap();
        let parts: Vec<&str> = real_line.split(' ').collect();
        let pkt_length = parts[parts.len()-1];
        let pkt_size = match f32::from_str(pkt_length) {
            Ok(p) => {
                if p == 0.0 {
                    continue;
                }
                p.ln()
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
        let note = scale[first + 24].clone();
        if pkt_size.floor() as u64 == 0 {
            continue;
        }
        tx.send((
            note,
            (pkt_size.ceil() as u64) * 1000000000 / 8,
            (1 + (clock / QUANTIZE_NS)) * QUANTIZE_NS
        )).unwrap();
    }
    // let wave = gen_wave(target_bytes, 440);
    // device.queue(&wave);
    // Start playback 

    // Play for 2 seconds 
    std::thread::sleep(Duration::from_millis(2000));

    // Device is automatically closed when dropped 
}