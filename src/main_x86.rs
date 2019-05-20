#![feature(duration_as_u128)]
extern crate image;
extern crate rayon;
extern crate termion;

use std::env;
use std::fs;
use std::io::{stdout, Write};
use std::{thread, time};
use termion::raw::IntoRawMode;

mod grid;
mod kernels;
mod sph;

const DT: f64 = 0.0005;
const WIDTH: u16 = 100;
const HEIGHT: u16 = 30;

#[cfg(target_arch = "x86_64")]
fn render_state(
    stdout: &mut std::io::Stdout,
    state: &sph::State,
    grid: &grid::Grid,
    debug: sph::SPHDebug,
) {
    let width = WIDTH;
    let height = HEIGHT;
    for y in 0..height {
        for x in 0..width {
            let density = sph::density(
                &state.particles,
                &grid,
                sph::MIN_X + x as f64 * (sph::MAX_X - sph::MIN_X) / width as f64,
                sph::MIN_Y + y as f64 * (sph::MAX_Y - sph::MIN_Y) / height as f64,
            );
            let mut norm_density = (9. * density / (debug.max_density)).round() as i32;
            if norm_density > 9 {
                norm_density = 9;
            }
            if norm_density < 0 {
                norm_density = 0;
            }
            if norm_density > 0 {
                write!(
                    stdout,
                    "{}{}",
                    termion::cursor::Goto(x + 1, y + 1),
                    norm_density as i32
                );
            } else {
                write!(stdout, "{}{}", termion::cursor::Goto(x + 1, y + 1), " ");
            }
        }
    }
    write!(
        stdout,
        "{}Max density: {}",
        termion::cursor::Goto(1, height + 1),
        debug.max_density
    );
    write!(
        stdout,
        "{}Max neighbours: {}",
        termion::cursor::Goto(1, height + 2),
        debug.n_neighbours
    );
    write!(
        stdout,
        "{}Frame time: {}",
        termion::cursor::Goto(1, height + 3),
        debug.frame_time
    );
    write!(
        stdout,
        "{}H: {}",
        termion::cursor::Goto(1, height + 4),
        debug.h
    );
}

fn render_png(state: &sph::State, grid: &grid::Grid, debug: sph::SPHDebug, frame: u32, size: u32) {
    fs::create_dir("output");
    let mut img = image::GrayImage::new(size, size);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let density = sph::density(
            &state.particles,
            &grid,
            x as f64 * (sph::MAX_X - sph::MIN_X) / size as f64,
            y as f64 * (sph::MAX_Y - sph::MIN_Y) / size as f64,
        );
        let mut norm_density = (255. * density / (debug.max_density)).round();
        if norm_density > 255.0 {
            norm_density = 255.0;
        }
        if norm_density < 0.0 {
            norm_density = 0.0;
        }
        *pixel = image::Luma([norm_density as u8]);
    }
    let filename = format!("output/image{:04}.png", frame);
    img.save(&filename).unwrap();
    println!("{}", filename);
}

enum Mode {
    Terminal,
    Image { size: u32 },
}

fn handle_args() -> Mode {
    let args: Vec<String> = env::args().collect();
    let mut mode = Mode::Terminal;
    if args.len() == 3 {
        if args[1] == "image" {
            mode = Mode::Image {
                size: args[2].parse().unwrap(),
            };
        }
    }
    return mode;
}

fn main() {
    let mut stdout = stdout(); //.into_raw_mode().unwrap();
                               //write!(stdout, "{}", termion::clear::All);
    let mut state = sph::create_initial_state();
    let mut frame = 0;
    let mode = handle_args();
    while true {
        let t1 = time::Instant::now();
        let (grid, debug) = sph::update_state(&mut state, DT, sph::SPHDebug::new());
        let frame_time = t1.elapsed().as_micros();
        match mode {
            Mode::Terminal => render_state(
                &mut stdout,
                &state,
                &grid,
                sph::SPHDebug {
                    frame_time,
                    ..debug
                },
            ),
            Mode::Image { size } => render_png(
                &state,
                &grid,
                sph::SPHDebug {
                    frame_time,
                    ..debug
                },
                frame,
                size,
            ),
        }
        frame += 1;
    }
}
