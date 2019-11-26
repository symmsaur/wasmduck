#[macro_use]
extern crate stdweb;
extern crate webgl_stdweb;

use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{self, INonElementParentNode, TypedArray};
use webgl_stdweb::{WebGLRenderingContext as GL, ANGLE_instanced_arrays, WebGLBuffer};

mod grid;
mod kernels;
mod math;
mod sph;

macro_rules! log {
    ($message:expr) => {
        js! {
            console.log(@{$message});
        }
    };
}


const DT: f64 = 0.0005;

struct Canvas {
    pub canvas: CanvasElement,
    pub ctx: GL,
    pub ext: ANGLE_instanced_arrays,
    pub offset_buffer: std::option::Option<WebGLBuffer>,
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

    let water_texture = ctx.create_texture();
    ctx.bind_texture(GL::TEXTURE_2D, water_texture.as_ref());

    let level = 0;
    let internal_format = GL::RGBA as i32;
    let width = 4;
    let height = 4;
    let border = 0;
    let src_format = GL::RGBA;
    let src_type = GL::UNSIGNED_BYTE;
    let pixel_buf = [
        // top row
        4u8, 4u8, 255u8, 16u8,
        4u8, 4u8, 255u8, 16u8,
        4u8, 4u8, 255u8, 16u8,
        4u8, 4u8, 255u8, 16u8,
        // first middle row
        4u8, 4u8, 255u8, 16u8,
        4u8, 4u8, 255u8, 64u8,
        4u8, 4u8, 255u8, 64u8,
        4u8, 4u8, 255u8, 32u8,
        // second middle row
        4u8, 4u8, 255u8, 16u8,
        4u8, 4u8, 255u8, 64u8,
        4u8, 4u8, 255u8, 64u8,
        4u8, 4u8, 255u8, 32u8,
        // bottom row
        4u8, 4u8, 255u8, 16u8,
        4u8, 4u8, 255u8, 16u8,
        4u8, 4u8, 255u8, 16u8,
        4u8, 4u8, 255u8, 16u8,
    ];
    let pixel = TypedArray::<u8>::from(&pixel_buf[..]);
    ctx.tex_image2_d(
        GL::TEXTURE_2D,
        level,
        internal_format,
        width,
        height,
        border,
        src_format,
        src_type,
        Some(&pixel),
    );

    // ctx.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
    // ctx.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);

    ctx.generate_mipmap(GL::TEXTURE_2D);

    let quad_buffer = ctx.create_buffer();
    ctx.bind_buffer(GL::ARRAY_BUFFER, quad_buffer.as_ref());

    let quad_vert_internal = [0.0, 0.0, 0.0,
                              1.0, 0.0, 0.0,
                              0.0, 1.0, 0.0,
                              1.0, 1.0, 0.0];

    let quad_vert_data = TypedArray::<f32>::from(&quad_vert_internal[..]).buffer();
    ctx.buffer_data_1(GL::ARRAY_BUFFER, Some(&quad_vert_data), GL::STATIC_DRAW);

    let index_buffer = ctx.create_buffer();
    ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, index_buffer.as_ref());
    let indices = [1u16, 0u16, 2u16,
                   2u16, 1u16, 3u16];
    let index_data = TypedArray::<u16>::from(&indices[..]).buffer();
    ctx.buffer_data_1(GL::ELEMENT_ARRAY_BUFFER, Some(&index_data), GL::STATIC_DRAW);

    let tex_coord_buffer = ctx.create_buffer();
    ctx.bind_buffer(GL::ARRAY_BUFFER, tex_coord_buffer.as_ref());
    let tex_coord_internal = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let tex_coord_data = TypedArray::<f32>::from(&tex_coord_internal[..]).buffer();
    ctx.buffer_data_1(GL::ARRAY_BUFFER, Some(&tex_coord_data), GL::STATIC_DRAW);


    let offset_buffer = ctx.create_buffer();
    ctx.bind_buffer(GL::ARRAY_BUFFER, offset_buffer.as_ref());
    let offset_data = TypedArray::<f32>::from(&[0.0; sph::N_PARTICLES as usize * 2][..]).buffer();
    ctx.buffer_data_1(GL::ARRAY_BUFFER, Some(&offset_data), GL::STATIC_DRAW);

    // Create vertex shader
    let vert_shader_code = r#"
            attribute vec3 position;
            attribute vec2 aTextureCoord;
            attribute vec2 offset;

            varying highp vec2 vTextureCoord;

            void main(void) {
                vec3 pos = position / 8.0 + vec3(offset, 0.0);
                gl_Position = vec4(pos, 1.0);
                vTextureCoord = aTextureCoord;
            }"#;
    let vert_shader = ctx.create_shader(GL::VERTEX_SHADER).unwrap();
    ctx.shader_source(&vert_shader, vert_shader_code);
    ctx.compile_shader(&vert_shader);

    log!("Vertex shader");
    log!(ctx.get_shader_info_log(&vert_shader));

    // Create fragment shader
    let frag_shader_code = r#"
            varying highp vec2 vTextureCoord;
            uniform sampler2D uSampler;
            void main(void) {
                gl_FragColor = texture2D(uSampler, vTextureCoord);
            }"#;
    let frag_shader = ctx.create_shader(GL::FRAGMENT_SHADER).unwrap();
    ctx.shader_source(&frag_shader, frag_shader_code);
    ctx.compile_shader(&frag_shader);

    log!("Fragment shader");
    log!(ctx.get_shader_info_log(&frag_shader));

    // Create shader program
    let shady_program = ctx.create_program().unwrap();
    ctx.attach_shader(&shady_program, &vert_shader);
    ctx.attach_shader(&shady_program, &frag_shader);
    ctx.link_program(&shady_program);

    // Associate attributes to shaders
    ctx.bind_buffer(GL::ARRAY_BUFFER, quad_buffer.as_ref());
    let pos = ctx.get_attrib_location(&shady_program, "position") as u32;
    ctx.vertex_attrib_pointer(pos, 3, GL::FLOAT, false, 0, 0);
    ctx.enable_vertex_attrib_array(pos);

    ctx.bind_buffer(GL::ARRAY_BUFFER, tex_coord_buffer.as_ref());
    let tex = ctx.get_attrib_location(&shady_program, "aTextureCoord") as u32;
    ctx.vertex_attrib_pointer(tex, 2, GL::FLOAT, false, 0, 0);
    ctx.enable_vertex_attrib_array(tex);

    let ext = ctx.get_extension::<ANGLE_instanced_arrays>().unwrap();
    ctx.bind_buffer(GL::ARRAY_BUFFER, offset_buffer.as_ref());
    let offs = ctx.get_attrib_location(&shady_program, "offset") as u32;
    ctx.vertex_attrib_pointer(offs, 2, GL::FLOAT, false, 0, 0);
    ctx.enable_vertex_attrib_array(offs);
    ext.vertex_attrib_divisor_angle(offs, 1);

    ctx.use_program(Some(&shady_program));
    ctx.enable(GL::BLEND);
    ctx.blend_func(GL::SRC_ALPHA, GL::ONE);

    ctx.clear_color(0.0, 0.0, 0.0, 1.0);

    ext.draw_elements_instanced_angle(GL::TRIANGLES, 6, GL::UNSIGNED_SHORT, 0, sph::N_PARTICLES as i32);

    let state = sph::create_initial_state();

    // ctx.viewport(0, 0, width as i32, height as i32);
    let canvas_holder = Canvas { canvas, ctx, ext, offset_buffer };
    main_loop(canvas_holder, state, 0.0);
}

fn main_loop(canvas: Canvas, mut state: sph::State, _dt: f64) {
    let (_grid, _debug) = sph::update_state(&mut state, DT, sph::SPHDebug::new());

    let mut offsets = [0.0; sph::N_PARTICLES as usize * 2];
    for i in 0..(sph::N_PARTICLES as usize) {
        let x = state.particles[i].x;
        let y = state.particles[i].y;
        offsets[2 * i] =
            ((((x - sph::MIN_X) / (sph::MAX_X - sph::MIN_X)) - 0.5) * 2.0) as f32;
        offsets[2 * i + 1] =
            (((-(y - sph::MIN_Y) / (sph::MAX_Y - sph::MIN_Y)) + 0.5) * 2.0) as f32;
    }
    let offset_data = TypedArray::<f32>::from(&offsets[..]).buffer();
    canvas.ctx.bind_buffer(GL::ARRAY_BUFFER, canvas.offset_buffer.as_ref());
    canvas
        .ctx
        .buffer_data_1(GL::ARRAY_BUFFER, Some(&offset_data), GL::STATIC_DRAW);
    canvas.ctx.clear(GL::COLOR_BUFFER_BIT);
    canvas.ext.draw_elements_instanced_angle(GL::TRIANGLES, 6, GL::UNSIGNED_SHORT, 0, sph::N_PARTICLES as i32);
    web::window().request_animation_frame(move |dt| {
        main_loop(canvas, state, dt);
    });
}
