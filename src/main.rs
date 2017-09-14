#[macro_use]
extern crate lazy_static;
extern crate sdl2;

use sdl2::audio::AudioSpecDesired;
use std::iter::repeat;
use std::io;
use std::io::{BufRead, Read};
use std::time::Duration;

fn gen_wave(bytes_to_write: i32, frequency: f32) -> Vec<i16> {
    // Generate a square wave
    let tone_volume = 1000i16;
    let period = 48000.0 / frequency;
    let sample_count = bytes_to_write;
    let mut result = Vec::new();
  
    for x in 0..sample_count {
        result.push(
                if (x / period as i32) % 2 == 0 {
                tone_volume
                }
                else {
                -tone_volume
                }
        );
    }
    result
}

static f_zero: f32 = 440.0;

fn note_freq(half_tone_offset: i32) -> f32 {
    lazy_static! {
        static ref A: f32 = (2.0 as f32).powf(1.0/12.0);
    }
    f_zero * A.powf(half_tone_offset as f32)
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let major_scale: Vec<i32> = vec![2, 2, 1, 2, 2, 2, 1];
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
        freq: Some(48000),
        channels: Some(1),
        // mono  -
        samples: None,
        // default sample size 
        };

    let device = audio_subsystem.open_queue::<i16, _>(None, &desired_spec).unwrap();

    println!("before resume");
    device.resume();
    println!("after resume");
    let target_bytes = 48000 * 4;
    for line in io::BufReader::new(io::stdin()).lines() {
        let real_line = line.unwrap();
        println!("{:?}", real_line);
        let parts: Vec<&str> = real_line.split(' ').collect();
        println!("{:?}", parts);
        let octets: Vec<&str> = parts[2].split('.').collect();
        let first = usize::from_str_radix(octets[0], 10).unwrap() / 8;
        println!("{:?}", first);
        let note = c_major_scale[first].clone();
        let wave = gen_wave(48000 / 32, note_freq(note));
        device.queue(&wave);
    }
    // let wave = gen_wave(target_bytes, 440);
    // device.queue(&wave);
    // Start playback 

    // Play for 2 seconds 
    std::thread::sleep(Duration::from_millis(2000));

    // Device is automatically closed when dropped 
}