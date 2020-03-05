extern crate num;

use std::thread;

use num::Complex;
use png;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

const MAX_ITERATIONS: u8 = 120;
const THREADS: u32 = 16;

fn mandelbrot(z: Complex<f64>, c: Complex<f64>, n: u8) -> Option<u8> {
    if z.norm() > 1000.0 {
        Some(n)
    } else if n < MAX_ITERATIONS {
        mandelbrot(z.powf(2.0) + c, c, n + 1)
    } else {
        None
    }
}

fn colour(n: u8) -> Vec<u8> {
    let c = ((n as f32 / MAX_ITERATIONS as f32) * 255.0) as u8;
    vec![c, c, c, 255]
}

fn translate(
    source_min: u32,
    source_max: u32,
    destination_min: f64,
    destination_max: f64,
    value: u32,
) -> f64 {
    let base_value = (value - source_min) as f64 / (source_max - source_min) as f64;
    return base_value * (destination_max - destination_min) + destination_min;
}

fn to_file(width: u32, height: u32, filename: &str) {
    let path = Path::new(filename);
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();

    let mut data = Vec::<u8>::new();
    let mut handles = vec![];

    let thread_height = height / THREADS;

    for t in 0..THREADS {
        let handle = thread::spawn(move || -> Vec<u8> {
            let mut result = Vec::<u8>::new();

            let y_range = (t * thread_height)..((t + 1) * thread_height);

            println!("Thread {} {:?}", t, y_range);

            for yy in y_range {
                let y = translate(0, height, -1.0, 1.0, yy);

                for xx in 0..width {
                    let x = translate(0, width, -2.0, 1.0, xx);

                    let m = mandelbrot(Complex::new(0.0, 0.0), Complex::new(x, y), 0);

                    let colour = match m {
                        Some(n) => colour(n),
                        None => vec![255, 255, 255, 255],
                    };
                    result.extend_from_slice(&colour);
                }
            }

            result
        });

        handles.push(handle);
    }

    for handle in handles {
        let line = handle.join().unwrap();
        data.extend_from_slice(&line);
    }

    writer.write_image_data(&data).unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        panic!("Requires filename as an argument")
    }

    let filename = &args[1];

    to_file(1600, 800, &filename);
}
