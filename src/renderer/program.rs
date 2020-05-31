use std::collections::HashMap;
use std::ffi::{CStr, CString};

use gl::types::{
    GLuint,
    GLint,
    GLchar,
};

use super::shader::Shader;
use crate::Resources;
// TODO: rename?
pub struct Program {
    id: GLuint,
    uniforms: HashMap<String, GLint> 
}

impl Program {
    // TODO: rename?
    pub fn set_used(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    // TODO: interface should take cstr
    pub fn set_i32(&mut self, name: &Cstr, value: i32) {
        if self.register_uniform(name) {
            unsafe {
                gl::Uniform1i(self.uniforms[name], value);
            }
        }
    }

    /// creates a program out of a folder path that contains both a fragment shader and vertex shader
    pub fn from_resources(res: &Resources, name: &str) -> Result<Program, String> {
        const POSSIBLE_EXT: [&str; 2] = [
            ".vert",
            ".frag",
        ];

        let shaders = POSSIBLE_EXT.iter()
            .map(|file_extension| {
                Shader::from_resources(res, &format!("{}{}", name, file_extension))
            })
            .collect::<Result<Vec<Shader>, String>>()?;

        Program::from_shaders(&shaders[..])
    }

    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl::CreateProgram() };

        for shader in shaders {
            unsafe { gl::AttachShader(program_id, shader.id()); }
        }

        unsafe { gl::LinkProgram(program_id); }

        let mut success: GLint = 1;
        unsafe {
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: GLint = 0;
            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = super::utils::create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl::GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut GLchar
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe { gl::DetachShader(program_id, shader.id()); }
        }

        // TODO: waste creating a vec every time just to have variables. Redesign this
        Ok(Program { id: program_id, uniforms: HashMap::new() })
    }

    // rename
    fn register_uniform(&mut self, name: &str) -> bool {
        if !self.uniforms.contains_key(name) {
            let uni_location = unsafe {
                // TODO: this section should be rewritten, iæm just a noob with String <-> Cstring
                use std::ffi::CString;
                let name_raw = match CString::new(name) {
                    Ok(str) => str.to_bytes_with_nul().as_ptr(),
                    Err(err) => panic!(err) // TODO: we don't need to panic here
                };

                gl::GetUniformLocation(self.id, name_raw as *const i8)
            };

            if uni_location != -1 {
                &self.uniforms.insert(name.to_string(), uni_location);
            } else {
                return false;
            }
        }

        return true;
    }
}


impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}