use std::ffi::{CStr, CString};
use std::ops::Deref;

use glutin::prelude::*;
use tracing::{debug, error};

pub mod gl {
    #![allow(clippy::all)]
    #![allow(unsafe_op_in_unsafe_fn)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

    pub use Gles2 as Gl;
}

pub mod egl {
    #![allow(clippy::all)]
    #![allow(unsafe_op_in_unsafe_fn)]
    #![allow(non_camel_case_types)]
    include!(concat!(env!("OUT_DIR"), "/egl_bindings.rs"));

    use std::os::raw;
    pub type khronos_utime_nanoseconds_t = raw::c_int;
    pub type khronos_uint64_t = u64;
    pub type khronos_ssize_t = isize;
    pub type EGLNativeDisplayType = *const raw::c_void;
    pub type EGLNativePixmapType = *const raw::c_void;
    pub type EGLNativeWindowType = *const raw::c_void;
    pub type EGLint = raw::c_int;
    pub type NativeDisplayType = *const raw::c_void;
    pub type NativePixmapType = *const raw::c_void;
    pub type NativeWindowType = *const raw::c_void;
}

pub struct Renderer {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    texture: gl::types::GLuint,
    gl: gl::Gl,
    egl: egl::Egl,
}

extern "system" fn callback(
    _source: u32,
    _gltype: u32,
    _id: u32,
    _severity: u32,
    length: i32,
    message: *const gl::types::GLchar,
    _user_param: *mut std::ffi::c_void,
) {
    let _result = std::panic::catch_unwind(move || unsafe {
        let slice = std::slice::from_raw_parts(message as *const u8, length as usize);
        let msg = String::from_utf8_lossy(slice);
        debug!("gl: {}", msg);
    });
}

impl Renderer {
    pub fn new(egl_display: &glutin::api::egl::display::Display) -> Self {
        unsafe {
            let egl = egl::Egl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                egl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            let gl = gl::Gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                // egl_display is also a GlDisplay
                egl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            if let Some(renderer) = get_gl_string(&gl, gl::RENDERER) {
                debug!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(&gl, gl::VERSION) {
                debug!("OpenGL Version {}", version.to_string_lossy());
            }

            if let Some(shaders_version) = get_gl_string(&gl, gl::SHADING_LANGUAGE_VERSION) {
                debug!("Shaders version on {}", shaders_version.to_string_lossy());
            }

            gl.DebugMessageCallback(Some(callback), std::ptr::null_mut());

            let vertex_shader = create_shader(&gl, gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE);
            let fragment_shader = create_shader(&gl, gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);

            let program = gl.CreateProgram();

            gl.AttachShader(program, vertex_shader);
            gl.AttachShader(program, fragment_shader);

            gl.LinkProgram(program);

            let mut status = std::mem::zeroed();
            gl.GetProgramiv(program, gl::LINK_STATUS, &mut status);
            debug!("glLinkProgram: {}", status);

            if status == 0 {
                let mut info_log: [i8; 2048] = std::mem::zeroed();
                let mut len = std::mem::zeroed();
                gl.GetProgramInfoLog(
                    program,
                    info_log.len() as _,
                    &mut len,
                    info_log.as_mut_ptr() as _,
                );
                error!("program: {:?}", CStr::from_ptr(info_log.as_mut_ptr() as _));

                gl.GetShaderInfoLog(
                    vertex_shader,
                    info_log.len() as _,
                    &mut len,
                    info_log.as_mut_ptr() as _,
                );
                error!("vertex: {:?}", CStr::from_ptr(info_log.as_mut_ptr() as _));

                gl.GetShaderInfoLog(
                    fragment_shader,
                    info_log.len() as _,
                    &mut len,
                    info_log.as_mut_ptr() as _,
                );
                error!("fragment: {:?}", CStr::from_ptr(info_log.as_mut_ptr() as _));
            }

            gl.UseProgram(program);

            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            let mut texture = std::mem::zeroed();
            gl.GenTextures(1, &mut texture);
            gl.BindTexture(gl::TEXTURE_2D, texture);

            let mut vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);

            Self {
                program,
                vao,
                vbo,
                texture,
                gl,
                egl,
            }
        }
    }

    pub fn prepare_texture(&self, extimage: egl::types::EGLImage) {
        unsafe {
            self.gl.BindTexture(gl::TEXTURE_2D, self.texture);
            self.gl
                .EGLImageTargetTexture2DOES(gl::TEXTURE_EXTERNAL_OES, extimage);
            self.gl.BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn draw(&self, tone_mapping_offset: f32, tone_mapping_scaling: f32) {
        unsafe {
            self.gl.UseProgram(self.program);

            self.gl.ActiveTexture(gl::TEXTURE0);
            self.gl.BindTexture(gl::TEXTURE_2D, self.texture);

            self.gl.Uniform1f(
                self.gl
                    .GetUniformLocation(self.program, b"toneMappingOffset\0".as_ptr() as _),
                tone_mapping_offset,
            );
            self.gl.Uniform1f(
                self.gl
                    .GetUniformLocation(self.program, b"toneMappingScaling\0".as_ptr() as _),
                tone_mapping_scaling,
            );

            self.gl.BindVertexArray(self.vao);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            self.gl.ClearColor(0.1, 0.1, 0.1, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
            self.gl.DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
            self.gl.Scissor(0, 0, width, height);
        }
    }

    pub fn egl(&self) -> &egl::Egl {
        &self.egl
    }
}

impl Deref for Renderer {
    type Target = gl::Gl;

    fn deref(&self) -> &Self::Target {
        &self.gl
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.program);
            self.gl.DeleteBuffers(1, &self.vbo);
            self.gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}

unsafe fn create_shader(
    gl: &gl::Gl,
    shader: gl::types::GLenum,
    source: &[u8],
) -> gl::types::GLuint {
    unsafe {
        let shader = gl.CreateShader(shader);
        gl.ShaderSource(
            shader,
            1,
            [source.as_ptr().cast()].as_ptr(),
            std::ptr::null(),
        );
        gl.CompileShader(shader);
        shader
    }
}

fn get_gl_string(gl: &gl::Gl, variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl.GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 300 es

#ifdef GL_ES
precision mediump float;
#endif

out vec2 tex;

void main()
{
	float idHigh = float(gl_VertexID >> 1);
	float idLow = float(gl_VertexID & int(1));

	float x = idHigh * 4.0 - 1.0;
	float y = idLow * 4.0 - 1.0;

	float u = idHigh * 2.0;
	float v = idLow * 2.0;

	gl_Position = vec4(x, y, 0.0, 1.0);
	tex = vec2(u, v);
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 300 es
#extension GL_OES_EGL_image_external : require

#ifdef GL_ES
precision lowp float;
#endif

uniform samplerExternalOES image;
uniform float toneMappingOffset;
uniform float toneMappingScaling;

vec3 toneMapping(vec3 color)
{
    return (color + vec3(toneMappingOffset)) * toneMappingScaling;
}

in vec2 tex;
layout(location = 0) out vec4 color;
void main()
{
	color = vec4(toneMapping(texture2D(image, tex).rgb), 1.0);
}
\0";
