use std::time::Instant;
use gax::{AccuracySynth, GaxVoice};

fn main() {
    println!("=== gax-rs Audio Performance Benchmarks ===");

    let pcm8_data = vec![64i8; 2048];
    let mut out_l = vec![0i8; 1024];
    let mut out_r = vec![0i8; 1024];
    let mut voice_1to1 = GaxVoice::new(0, 0x1000, 128, 64);

    let iters = 100_000;

    // 1. AccuracySynth::mix_voice_pcm8 (1:1 pitch)
    for _ in 0..1000 {
        voice_1to1.position_fixed = 0;
        voice_1to1.is_active = true;
        AccuracySynth::mix_voice_pcm8(&pcm8_data, &mut voice_1to1, &mut out_l, &mut out_r);
    }
    let start = Instant::now();
    for _ in 0..iters {
        voice_1to1.position_fixed = 0;
        voice_1to1.is_active = true;
        AccuracySynth::mix_voice_pcm8(&pcm8_data, &mut voice_1to1, &mut out_l, &mut out_r);
    }
    let elapsed = start.elapsed();
    let ns_per_iter = elapsed.as_nanos() as f64 / iters as f64;
    let msamples_per_sec = ((iters * 1024) as f64 / elapsed.as_secs_f64()) / 1_000_000.0;
    println!(
        "AccuracySynth::mix_voice_pcm8 (1:1 pitch, 1024 frames): {:.2} ns/iter ({:.2} MSamples/sec)",
        ns_per_iter, msamples_per_sec
    );

    // 2. AccuracySynth::mix_voice_pcm8 (resampled pitch 0x1500)
    let mut voice_resample = GaxVoice::new(0, 0x1500, 128, 64);
    for _ in 0..1000 {
        voice_resample.position_fixed = 0;
        voice_resample.is_active = true;
        AccuracySynth::mix_voice_pcm8(&pcm8_data, &mut voice_resample, &mut out_l, &mut out_r);
    }
    let start = Instant::now();
    for _ in 0..iters {
        voice_resample.position_fixed = 0;
        voice_resample.is_active = true;
        AccuracySynth::mix_voice_pcm8(&pcm8_data, &mut voice_resample, &mut out_l, &mut out_r);
    }
    let elapsed = start.elapsed();
    let ns_per_iter = elapsed.as_nanos() as f64 / iters as f64;
    let msamples_per_sec = ((iters * 1024) as f64 / elapsed.as_secs_f64()) / 1_000_000.0;
    println!(
        "AccuracySynth::mix_voice_pcm8 (resampled 0x1500, 1024 frames): {:.2} ns/iter ({:.2} MSamples/sec)",
        ns_per_iter, msamples_per_sec
    );
}
