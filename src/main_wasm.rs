#[macro_use]
extern crate stdweb;
extern crate webgl_stdweb;

use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{self, INonElementParentNode, TypedArray};
use webgl_stdweb::{WebGLRenderingContext as GL};

mod grid;
mod kernels;
mod math;
mod sph;

const DT: f64 = 0.0005;

struct Canvas {
    pub canvas: CanvasElement,
    pub ctx: GL,
}

fn main() {
    let canvas: CanvasElement = web::document()
        .get_element_by_id("win")
        .unwrap()
        .try_into()
        .unwrap();
    let width = canvas.width();
    let height = canvas.height();
    let ctx: GL = canvas.get_context().unwrap();

    let vertices = TypedArray::<f32>::from(&[0.0; (sph::N_PARTICLES + 1) as usize * 3][..]).buffer();
    let vertex_buffer = ctx.create_buffer().unwrap();
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
    ctx.buffer_data_1(GL::ARRAY_BUFFER, Some(&vertices), GL::STATIC_DRAW);

    let mut vinst = [0u16; (sph::N_PARTICLES + 1) as usize];
    for i in 0..(sph::N_PARTICLES+1) {
        vinst[i as usize] = i as u16;
    }
    let indices = TypedArray::<u16>::from(&vinst[..]).buffer();
    let index_buffer = ctx.create_buffer().unwrap();
    ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
    ctx.buffer_data_1(GL::ELEMENT_ARRAY_BUFFER, Some(&indices), GL::STATIC_DRAW);

    // Create vertex shader
    let vert_shader_code = r#"
        attribute vec3 position;

        void main(void) {
            gl_Position = vec4(position.xy, 0., 1.);
            if(position.z > 0.1)
            {
                gl_PointSize = 100.;
            }
            else
            {
                gl_PointSize = 2.;
            }

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
    ctx.vertex_attrib_pointer(pos, 3, GL::FLOAT, false, 0, 0);
    ctx.enable_vertex_attrib_array(pos);

    ctx.use_program(Some(&shady_program));

    let state = sph::create_initial_state();

    ctx.viewport(0, 0, width as i32, height as i32);
    let canvas_holder = Canvas {
        canvas,
        ctx,
    };
    main_loop(canvas_holder, state, 0.0);
}

// macro_rules! log {
//     ($message:expr) => {
//         js! {
//             console.log(@{$message});
//         }
//     };
// }

fn main_loop(canvas: Canvas, mut state: sph::State, _dt: f64) {
    let (_grid, _debug) = sph::update_state(&mut state, DT, sph::SPHDebug::new());

    let mut vertices_array = [0.0f32; (sph::N_PARTICLES + 1) as usize * 3];
    for i in 0..(sph::N_PARTICLES as usize) {
        let x = state.particles[i].x;
        let y = state.particles[i].y;
        vertices_array[3+3 * i] =
            ((((x - sph::MIN_X) / (sph::MAX_X - sph::MIN_X)) - 0.5) * 2.0) as f32;
        vertices_array[3+3 * i + 1] =
            (((-(y - sph::MIN_Y) / (sph::MAX_Y - sph::MIN_Y)) + 0.5) * 2.0) as f32;
        vertices_array[3+3 * i + 2] = 0.;
    }

    vertices_array[0] =
            ((((state.duck.x - sph::MIN_X) / (sph::MAX_X - sph::MIN_X)) - 0.5) * 2.0) as f32;
    vertices_array[1] =
            (((-(state.duck.y - sph::MIN_Y) / (sph::MAX_Y - sph::MIN_Y)) + 0.5) * 2.0) as f32;
    vertices_array[2] = 1.0;
    let vertices = TypedArray::<f32>::from(&vertices_array[..]).buffer();
    canvas
        .ctx
        .buffer_data_1(GL::ARRAY_BUFFER, Some(&vertices), GL::STATIC_DRAW);
    canvas
        .ctx
        .draw_elements(GL::POINTS, (sph::N_PARTICLES + 1) as i32, GL::UNSIGNED_SHORT, 0);
    web::window().request_animation_frame(move |dt| {
        main_loop(canvas, state, dt);
    });
}
