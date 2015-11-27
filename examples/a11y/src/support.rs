use std::mem;
use glutin;
use gleam::gl;

pub fn load(window: &glutin::Window) {
    gl::load_with(|ptr| window.get_proc_address(ptr) as *const _);

    let version = gl::get_string(gl::VERSION);

    println!("OpenGL version {}", version);

    let vs = gl::create_shader(gl::VERTEX_SHADER);
    gl::shader_source(vs, &[&VS_SRC]);
    gl::compile_shader(vs);

    let fs = gl::create_shader(gl::FRAGMENT_SHADER);
    gl::shader_source(fs, &[&FS_SRC]);
    gl::compile_shader(fs);

    let program = gl::create_program();
    gl::attach_shader(program, vs);
    gl::attach_shader(program, fs);
    gl::link_program(program);
    gl::use_program(program);

    let vb = gl::gen_buffers(1);
    gl::bind_buffer(gl::ARRAY_BUFFER, vb[0]);
    gl::buffer_data(gl::ARRAY_BUFFER, &VERTEX_DATA, gl::STATIC_DRAW);

    if gl::BindVertexArray::is_loaded() {
        let vao = gl::gen_vertex_arrays(1);
        gl::bind_vertex_array(vao[0]);
    }

    let pos_attrib = gl::get_attrib_location(program, "position");
    let color_attrib = gl::get_attrib_location(program, "color");
    gl::vertex_attrib_pointer(pos_attrib as gl::types::GLuint, 2, gl::FLOAT, false,
                              5 * mem::size_of::<f32>() as gl::types::GLsizei,
                              0);
    gl::vertex_attrib_pointer(color_attrib as gl::types::GLuint, 3, gl::FLOAT, false,
                              5 * mem::size_of::<f32>() as gl::types::GLsizei,
                              (2 * mem::size_of::<f32>()) as u32);
    gl::enable_vertex_attrib_array(pos_attrib as gl::types::GLuint);
    gl::enable_vertex_attrib_array(color_attrib as gl::types::GLuint);
}

pub fn draw_frame(color: (f32, f32, f32, f32)) {
    gl::clear_color(color.0, color.1, color.2, color.3);
    gl::clear(gl::COLOR_BUFFER_BIT);

    gl::draw_arrays(gl::TRIANGLES, 0, 3);
}

static VERTEX_DATA: [f32; 15] = [
    -0.5, -0.5, 1.0, 0.0, 0.0,
    0.0, 0.5, 0.0, 1.0, 0.0,
    0.5, -0.5, 0.0, 0.0, 1.0
];

const VS_SRC: &'static [u8] = b"
#version 100
precision mediump float;
attribute vec2 position;
attribute vec3 color;
varying vec3 v_color;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_color = color;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 100
precision mediump float;
varying vec3 v_color;
void main() {
    gl_FragColor = vec4(v_color, 1.0);
}
\0";
