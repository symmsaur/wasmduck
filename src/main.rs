extern crate stdweb;

use stdweb::web::html_element::CanvasElement;
use stdweb::web::{self, INonElementParentNode, CanvasRenderingContext2d};
use stdweb::unstable::TryInto;

mod sph;


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
    for y in 0..height {
        for x in 0..width {
            let density = sph::density(x as f64 / width as f64, y as f64 / height as f64);
            let mut norm_density = (255. * (density - 0.6) / (0.7 - 0.6)).round() as i32;
            if norm_density > 255 {
                norm_density = 255;
            }
            if norm_density < 0 {
                norm_density = 0;
            }
            // let index = (y * width + x) * 4;
            ctx.set_fill_style_color(&format!("rgb({}, 0, 0)", norm_density));
            ctx.fill_rect(x as f64, y as f64, 1.0, 1.0);
        }
    }
}
