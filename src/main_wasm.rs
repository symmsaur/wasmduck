#[macro_use]
extern crate stdweb;
extern crate webgl_stdweb;

use stdweb::web::html_element::CanvasElement;
use stdweb::web::{self, INonElementParentNode, TypedArray};
use stdweb::unstable::TryInto;
use webgl_stdweb::{WebGLRenderingContext as GL, WebGLBuffer};

mod kernels;
mod math;
mod sph;
mod grid;

const DT: f64 = 0.0005;

struct Canvas {
    pub canvas: CanvasElement,
    pub ctx: GL,
    index_buffer: WebGLBuffer,
    width: u32,
    height: u32,
}

fn main() {
    let canvas: CanvasElement = web::document()
        .get_element_by_id("win")
        .unwrap()
        .try_into()
        .unwrap();
    let width = canvas.width();
    let height = canvas.height();
    let ctx: GL = canvas
        .get_context().unwrap();

    let vertices = TypedArray::<f32>::from(&[0.0; sph::N_PARTICLES as usize * 2][..]).buffer();
    let vertex_buffer = ctx.create_buffer().unwrap();
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
    ctx.buffer_data_1(GL::ARRAY_BUFFER, Some(&vertices), GL::STATIC_DRAW);

    let mut vinst = [0u16; sph::N_PARTICLES as usize];
    for i in 0..sph::N_PARTICLES {
        vinst[i as usize] = i as u16;
    }
    let indices = TypedArray::<u16>::from(&vinst[..]).buffer();
    let index_buffer = ctx.create_buffer().unwrap();
    ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
    ctx.buffer_data_1(GL::ELEMENT_ARRAY_BUFFER, Some(&indices), GL::STATIC_DRAW);

    // Create vertex shader
    let vert_shader_code = r#"
        attribute vec2 position;

        void main(void) {
            gl_Position = vec4(position, 0., 1.);
            gl_PointSize = 2.;
        }"#;
    let vert_shader = ctx.create_shader(GL::VERTEX_SHADER).unwrap();
    ctx.shader_source(&vert_shader, vert_shader_code);
    ctx.compile_shader(&vert_shader);

    // Create fragment shader
    let frag_shader_code = r#"
        void main(void) {
            gl_FragColor = vec4(0.0, 0.0, 1.0, 1.0);
        }"#;
    let frag_shader = ctx.create_shader(GL::FRAGMENT_SHADER).unwrap();
    ctx.shader_source(&frag_shader, frag_shader_code);
    ctx.compile_shader(&frag_shader);

    // Create shader program
    let shady_program = ctx.create_program().unwrap();
    ctx.attach_shader(&shady_program, &vert_shader);
    ctx.attach_shader(&shady_program, &frag_shader);
    ctx.link_program(&shady_program);

    // Associate attributes to shaders
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
    let pos = ctx.get_attrib_location(&shady_program, "position") as u32;
    ctx.vertex_attrib_pointer(pos, 2, GL::FLOAT, false, 0, 0);
    ctx.enable_vertex_attrib_array(pos);

    ctx.use_program(Some(&shady_program));

    let state = sph::create_initial_state();

    let canvas_holder = Canvas {
        canvas,
        ctx,
        index_buffer,
        width,
        height,
    };
    main_loop(canvas_holder, state, 0.0);
}

macro_rules! log {
    ($message:expr) =>
    (js! {
        console.log(@{$message});
    })
}

fn main_loop(canvas: Canvas, mut state: Vec<sph::Particle>, _dt: f64) {
    let (grid, debug) = sph::update_state(&mut state, DT, sph::SPHDebug::new());
    canvas.ctx.enable(GL::DEPTH_TEST);
    canvas.ctx.depth_func(GL::LEQUAL);
    canvas.ctx.clear_color(0.0, 0.0, 0.0, 1.0);
    canvas.ctx.clear_depth(1.0);

    canvas.ctx.viewport(0, 0, canvas.width as i32, canvas.height as i32);
    canvas.ctx.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

    let mut vertices_array = [0.0f32; sph::N_PARTICLES as usize * 2];
    for i in 0..(sph::N_PARTICLES as usize) {
        let x = state[i].x;
        let y = state[i].y;
        vertices_array[2 * i] = ((((x - sph::MIN_X) / (sph::MAX_X - sph::MIN_X)) - 0.5) * 2.0) as f32;
        vertices_array[2 * i + 1] = (((-(y - sph::MIN_Y) / (sph::MAX_Y - sph::MIN_Y)) + 0.5) * 2.0) as f32;
    }
    let vertices = TypedArray::<f32>::from(&vertices_array[..]).buffer();
    canvas.ctx.buffer_data_1(GL::ARRAY_BUFFER, Some(&vertices), GL::STATIC_DRAW);
    canvas.ctx.draw_elements(GL::POINTS, sph::N_PARTICLES as i32, GL::UNSIGNED_SHORT, 0);
    web::window().request_animation_frame(move |dt| {
        main_loop(canvas, state, dt);
    });
}
