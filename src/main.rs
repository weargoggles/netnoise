#[macro_use]
extern crate lazy_static;
extern crate sdl2;

use sdl2::audio::AudioSpecDesired;
use std::iter::repeat;
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
    let c3 = middle_c - 12;

    let mut c_major_scale = vec![c3];
    c_major_scale = major_scale_iter.take(24).fold(
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

    let target_bytes = 48000 * 4;
    for note in c_major_scale.iter() {
        let note = note.clone();
        let wave = gen_wave(48000 / 16, note_freq(note));
        device.queue(&wave);
    }
    // let wave = gen_wave(target_bytes, 440);
    // device.queue(&wave);
    // Start playback 
    device.resume();

    // Play for 2 seconds 
    std::thread::sleep(Duration::from_millis(2000));

    // Device is automatically closed when dropped 
}