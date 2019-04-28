extern crate time;
extern crate termion;
extern crate rayon;
extern crate image;
extern crate circular_queue;

use termion::raw::IntoRawMode;
use std::io::{Write, stdout};
use time::{SteadyTime, Duration};
use std::env;
use std::fs;

mod kernels;
mod math;
mod sph;
mod grid;

const DT: f64 = 0.0005;
const WIDTH: u16 = 100;
const HEIGHT: u16 = 30;

#[cfg(target_arch="x86_64")]
fn render_state(stdout: &mut std::io::Stdout, state: &Vec<sph::Particle>, grid: &grid::Grid, debug: sph::SPHDebug) {
    let width = WIDTH;
    let height = HEIGHT;
    for y in 0..height {
        for x in 0..width {
            let density = sph::density(&state, &grid, x as f64 * 5.0 / width as f64, y as f64 * 5.0 / height as f64);
            let mut norm_density = (9. * density / (debug.max_density)).round() as i32;
            if norm_density > 9 {
                norm_density = 9;
            }
            if norm_density < 0 {
                norm_density = 0;
            }
            if norm_density > 0 {
                write!(stdout, "{}{}", termion::cursor::Goto(x + 1, y + 1), norm_density as i32);
            }
            else {
                write!(stdout, "{}{}", termion::cursor::Goto(x + 1, y + 1), " ");
            }
        }
    }
    write!(stdout, "{}Max density: {}", termion::cursor::Goto(1, height + 1), debug.max_density);
    write!(stdout, "{}Max neighbours: {}", termion::cursor::Goto(1, height + 2), debug.n_neighbours);
    write!(stdout, "{}Frame time: {}", termion::cursor::Goto(1, height + 3), debug.frame_time);
    write!(stdout, "{}Frame rate: {}   ", termion::cursor::Goto(1, height + 4), debug.frame_rate);
    write!(stdout, "{}Grid time: {}", termion::cursor::Goto(1, height + 5), debug.grid_time);
    write!(stdout, "{}Update density time: {}", termion::cursor::Goto(1, height + 6), debug.update_density_time);
    write!(stdout, "{}Calculate forces time: {}", termion::cursor::Goto(1, height + 7), debug.calculate_forces_time);
    write!(stdout, "{}Average force: {}", termion::cursor::Goto(1, height + 8), debug.average_force);
    write!(stdout, "{}H: {}", termion::cursor::Goto(1, height + 9), debug.h);
}

fn render_png(state: &Vec<sph::Particle>, grid: &grid::Grid, debug: sph::SPHDebug, frame: u32, size: u32) {
    fs::create_dir("output");
    let mut img = image::GrayImage::new(size, size);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
            let density = sph::density(&state, &grid, x as f64 * 5.0 / size as f64, y as f64 * 5.0 / size as f64);
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
    Image { size: u32 }
}

fn handle_args() -> Mode {
    let args: Vec<String> = env::args().collect();
    let mut mode = Mode::Terminal;
    if args.len() == 3 {
        if args[1] == "image" {
            mode = Mode::Image { size: args[2].parse().unwrap() };
        }
    }
    return mode;
}

fn main() {
    let mut stdout = stdout();//.into_raw_mode().unwrap();
    //write!(stdout, "{}", termion::clear::All);
    let mut state = sph::create_initial_state();
    let mut frame = 0;
    let mode = handle_args();
    let mut update_density_time_queue = circular_queue::CircularQueue::with_capacity(10000);
    let mut calculate_forces_time_queue = circular_queue::CircularQueue::with_capacity(10000);

    let mut last_frame_for_calc = 0;
    let mut frame_rate_calc_time = SteadyTime::now();
    let mut frame_rate = 0;

    loop {
        let t1 = SteadyTime::now();
        if t1 - frame_rate_calc_time > Duration::seconds(10) {
            frame_rate_calc_time = t1;
            frame_rate = (frame - last_frame_for_calc)/10;
            last_frame_for_calc = frame;
        }

        let (grid, debug) = sph::update_state(&mut state, DT, sph::SPHDebug::new());
        update_density_time_queue.push(debug.update_density_time);
        calculate_forces_time_queue.push(debug.calculate_forces_time);

        let frame_time = SteadyTime::now()-t1;
        let debug = sph::SPHDebug {
            frame_time: frame_time,
            frame_rate: frame_rate,
            update_density_time: update_density_time_queue.iter().fold(Duration::zero(), |accumulate, &x| accumulate+x) / update_density_time_queue.len() as i32,
            calculate_forces_time: calculate_forces_time_queue.iter().fold(Duration::zero(), |accumulate, &x| accumulate+x) / calculate_forces_time_queue.len() as i32,
            .. debug};

        match mode {
            Mode::Terminal => render_state(&mut stdout, &state, &grid, debug),
            Mode::Image { size } => render_png(&state, &grid, debug, frame, size)
        }
        frame += 1;
    }
}
