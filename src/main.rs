#[macro_use]
extern crate stdweb;

use stdweb::web::html_element::CanvasElement;
use stdweb::web::{self, INonElementParentNode, CanvasRenderingContext2d};
use stdweb::unstable::TryInto;
use std::rc::Rc;
use std::cell::RefCell;

mod kernels;
mod math;
mod sph;

const DT: f64 = 0.01;
struct Canvas {
    pub canvas: CanvasElement,
    pub ctx: CanvasRenderingContext2d,
    width: u32,
    height: u32
}

fn main() {
    let tmp = web::document()
        .get_element_by_id("win")
        .unwrap();
    let canvas: CanvasElement = tmp
        .try_into()
        .unwrap();
    let width = canvas.width();
    let height = canvas.height();
    let ctx: CanvasRenderingContext2d = canvas
        .get_context().unwrap();
    let canvasHolder = Canvas {
        canvas,
        ctx,
        width,
        height
    };
    let mut state = sph::create_initial_state();
    main_loop(Rc::new(canvasHolder), Rc::new(RefCell::new(state)));
}

fn main_loop(canvas: Rc<Canvas>, state: Rc<RefCell<Vec<sph::Particle>>>) {
    let max_density = sph::update_density(&mut state.borrow_mut());
    sph::update_state(&mut state.borrow_mut(), DT);
    for y in 0..canvas.height {
        for x in 0..canvas.width {
            let density = sph::density(&state.borrow(), x as f64 * 5.0 / canvas.width as f64, y as f64 * 5.0 / canvas.height as f64);
            let mut norm_density = (255. * density / (max_density)).round() as i32;
            if norm_density > 255 {
                norm_density = 255;
            }
            if norm_density < 0 {
                norm_density = 0;
            }
            // let index = (y * width + x) * 4;
            canvas.ctx.set_fill_style_color(&format!("rgb({}, 0, 0)", norm_density));
            canvas.ctx.fill_rect(x as f64, y as f64, 1.0, 1.0);
        }
    }
    web::set_timeout(move || {
        main_loop(canvas.clone(), state.clone());
    }, 30);
}