use std;

const ATTACK: f32 = 0.05;
const DECAY: f32 = 0.0;
const SUSTAIN: f32 = 1.0;
const RELEASE: f32 = 0.5;

pub fn adsr(t: f32, total: f32) -> f32 {
    // 1.0
    if t < total * ATTACK {
        t / (total * ATTACK)
    } else if t < total * (1.0 - RELEASE) {
        SUSTAIN
    } else {
        (total - t) / (total * RELEASE)
    }
}

pub fn gen_wave(samples_to_write: i32, frequency: f32, sample_rate: i32) -> Vec<i16> {
    // Generate a square wave
    let sample_count = samples_to_write;
    let mut result = Vec::new();
  
    for t in 0..sample_count {
        let t_: f32 = (t as f32) / (sample_rate as f32);
        result.push(
                (8000.0 * adsr(t as f32, sample_count as f32) * (
                    t_ * (2.0 * (std::f32::consts::PI as f32)) * frequency
                ).sin()) as i16
        );
    }
    result
}