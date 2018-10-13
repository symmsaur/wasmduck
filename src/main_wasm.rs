#[macro_use]
extern crate stdweb;
extern crate rayon;

use stdweb::web::html_element::CanvasElement;
use stdweb::web::{self, INonElementParentNode, CanvasRenderingContext2d};
use stdweb::unstable::TryInto;

mod kernels;
mod math;
mod sph;
mod grid;

const DT: f64 = 0.0005;

struct Canvas {
    pub canvas: CanvasElement,
    pub ctx: CanvasRenderingContext2d,
    width: u32,
    height: u32
}

fn main() {
    let canvas: CanvasElement = web::document()
        .get_element_by_id("win")
        .unwrap()
        .try_into()
        .unwrap();
    let width = canvas.width();
    let height = canvas.height();
    let ctx: CanvasRenderingContext2d = canvas
        .get_context().unwrap();
    let canvas_holder = Canvas {
        canvas,
        ctx,
        width,
        height
    };
    let state = sph::create_initial_state();
    main_loop(canvas_holder, state, 0.0);
}

fn main_loop(canvas: Canvas, mut state: Vec<sph::Particle>, _dt: f64) {
    let (grid, debug) = sph::update_state(&mut state, DT, sph::SPHDebug::new());
    canvas.ctx.set_fill_style_color("rgb(0, 0, 0)");
    canvas.ctx.fill_rect(0.0, 0.0, canvas.width as f64, canvas.height as f64);
    for particle in &state {
        canvas.ctx.set_fill_style_color(&format!("rgb(0, 0, {})", 255));
        canvas.ctx.fill_rect(particle.x * canvas.width as f64 / 5.0, particle.y * canvas.height as f64 / 5.0, 5.0, 5.0);
    }
    let n = debug.n_neighbours as u32;
    js! {
        console.log(@{n});
    }
    web::window().request_animation_frame(move |dt| {
        main_loop(canvas, state, dt);
    });
}
