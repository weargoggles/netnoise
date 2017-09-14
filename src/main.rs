#[macro_use]
extern crate lazy_static;
extern crate sdl2;
extern crate time;

use sdl2::audio::AudioSpecDesired;
use std::iter::repeat;
use std::io;
use std::io::{BufRead, Read};
use std::str::FromStr;
use std::time::Duration;

const ATTACK: f32 = 0.05;
const DECAY: f32 = 0.0;
const SUSTAIN: f32 = 1.0;
const RELEASE: f32 = 0.75;

fn adsr(t: f32, total: f32) -> f32 {
    // 1.0
    if t < total * ATTACK {
        t / (total * ATTACK)
    } else if t < total * (1.0 - RELEASE) {
        1.0
    } else {
        (total - t) / (total * RELEASE)
    }
}

fn gen_wave(bytes_to_write: i32, frequency: f32) -> Vec<i16> {
    // Generate a square wave
    let sample_count = bytes_to_write;
    let mut result = Vec::new();
  
    for t in 0..sample_count {
        result.push(
                (8000.0 * adsr(t as f32, sample_count as f32) * (
                    ((t as f32) / SAMPLE_RATE as f32) * (2.0 * (std::f32::consts::PI as f32)) * frequency
                ).sin()) as i16
        );
    }
    result
}

static F_ZERO: f32 = 440.0;
static SAMPLE_RATE: i32 = 44100;


fn note_freq(half_tone_offset: i32) -> f32 {
    lazy_static! {
        static ref A: f32 = (2.0 as f32).powf(1.0/12.0);
    }
    F_ZERO * A.powf(half_tone_offset as f32)
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let major_scale: Vec<i32> = vec![2, 2, 1, 2, 1, 2, 1];
    let major_scale_iter = major_scale.iter().cycle();
    let middle_c: i32 = -9;
    let c0 = middle_c - 48;

    let mut c_major_scale = vec![c0];
    c_major_scale = major_scale_iter.take(63).fold(
        c_major_scale,
        |mut seq: Vec<i32>, offset: &i32| {
            let last = seq.pop().unwrap();
            let next = last + offset;
            seq.push(last);
            seq.push(next);
            seq
        }
    );
    println!("{:?}", c_major_scale);


    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE),
        channels: Some(1),
        // mono  -
        samples: Some(32768),
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
                p.log(2.0)
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
        let first = usize::from_str_radix(octets[3], 10).unwrap() / 8;
        let note = c_major_scale[first + 24].clone();
        let note_length_bytes = (SAMPLE_RATE / (64 / pkt_size.floor() as i32)) as i32;
        let note_length_ns = ((note_length_bytes as u64) / 44100) * 1000000000; 
        if note_length_ns > 50000000 {
            continue;
        }
        if bufuntil - now < 2000000000 {
            let wave = gen_wave(note_length_bytes, note_freq(note));
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