#[allow(unused_imports)]
extern crate gl;

use gl::types::*;

use std::ffi::CString;
use std::ptr;
use std::str;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

// Shader loader from
// https://github.com/bjz/gl-rs/blob/62b66a85e4c2c0d88fb916040bc462702c8e0346/gl/examples/static_triangle.rs
pub fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8"));
        }
    }
    return shader
}

pub fn compile_shader_file(file: &str, ty: GLenum) -> GLuint {
    let path = Path::new(file);
    let mut file = File::open(&path).unwrap();
    let mut source = String::new();
    let len = file.read_to_string(&mut source).unwrap();
    println!("Read in shader file {} of {} bytes", path.display(), len);
    return compile_shader(&source, ty);
}

pub fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    let program;
    unsafe {
        program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(&buf).ok().expect("ProgramInfoLog not valid utf8"));
        }
    }
    return program
}
