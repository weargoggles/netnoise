
static F_ZERO: f32 = 440.0;

lazy_static! {
    pub static ref IONIAN: Vec<i32> = vec![2, 2, 1, 2, 2, 2, 1];
    pub static ref HARMONIC_MAJOR: Vec<i32> = vec![2, 2, 1, 2, 1, 2, 1];
    pub static ref AEOLIAN: Vec<i32> = vec![2, 1, 2, 2, 1, 2, 2];
    pub static ref PHRYGIAN_DOMINANT: Vec<i32> = vec![1, 3, 1, 2, 1, 2, 2];
}

fn note_freq(half_tone_offset: i32) -> f32 {
    lazy_static! {
        static ref A: f32 = (2.0 as f32).powf(1.0/12.0);
    }
    F_ZERO * A.powf(half_tone_offset as f32)
}

pub fn scale(mode: Vec<i32>, tonic: i32) -> Vec<f32> {
    let scale_iter = mode.iter().cycle();

    let mut scale = vec![tonic]; //  in G
    scale_iter.take(63).fold(
        scale,
        |mut seq: Vec<i32>, offset: &i32| {
            let last = seq.pop().unwrap();
            let next = last + offset;
            seq.push(last);
            seq.push(next);
            seq
        }
    ).iter().map(|&n| note_freq(n)).collect()
}