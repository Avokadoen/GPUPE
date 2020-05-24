extern crate gl;
extern crate sdl2;
extern crate image;

use sdl2::event::Event;
use std::path::Path;

mod renderer;
mod resources;

use resources::Resources;
use renderer::{
    program::Program, 
    shader::Shader
};

fn main() {
    let res = Resources::from_relative_path(Path::new("assets")).unwrap();

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    {
        // Specify context version
        // currently we hardcode Opengl Core 4.5
        let gl_attr = video_subsystem.gl_attr();

        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 5);
    }

    let window_x: u32 = 512;
    let window_y: u32 = 512;

    let window = video_subsystem
        .window("GPUPE prototype", window_x, window_y)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    // keep context alive with varible
    let _gl_context = window.gl_create_context().unwrap();
    let _gl_load_with = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);
    let _gl_viewport_load_with = gl::Viewport::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

    unsafe {
        gl::Viewport(0, 0, window_x as i32, window_y as i32); // set viewport
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    // TODO: if we want chunks, then this should be generalized (buffers)
    // TODO: rename: triangle -> default
    // create quad data
    let triangle_program = Program::from_resources(&res, "shaders/triangle").unwrap();

    let vertices: Vec<f32> = vec![
    //   x,    y    z,   u,   v   
        -1.0, -1.0, 0.0, 0.0, 0.0, // bottom left
         1.0,  1.0, 0.0, 1.0, 1.0, // top right
        -1.0,  1.0, 0.0, 0.0, 1.0, // top left
         1.0, -1.0, 0.0, 1.0, 0.0  // bottom right
    ];

    let mut v_vbo: gl::types::GLuint = 0;
    unsafe {
        gl::GenBuffers(1, &mut v_vbo);
    }
    let v_vbo: gl::types::GLuint = v_vbo;

    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, v_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, // target
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
            vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
            gl::STATIC_DRAW, // usage
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0); // unbind the buffer
    }

    let indices: Vec<u32> = vec![
        0, 1, 2,
        0, 1, 3
    ];

    let mut i_vbo: gl::types::GLuint = 0;
    unsafe {
        gl::GenBuffers(1, &mut i_vbo);
    }
    let i_vbo: gl::types::GLuint = i_vbo;

    unsafe {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, i_vbo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER, // target
            (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr, // size of data in bytes
            indices.as_ptr() as *const gl::types::GLvoid, // pointer to data
            gl::STATIC_DRAW, // usage
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0); // unbind the buffer
    }

    // TODO: Handle error, we don't really even need a texture loading yet. just a image buffer that we will write to
    // let rust_image = res.load_image("textures/water_test.png")
    //     .unwrap()
    //     .into_rgba();
    
    // let mut texture: gl::types::GLuint = 0;
    // unsafe {
    //     gl::GenTextures(1, &mut texture);
 
    //     gl::BindTexture(gl::TEXTURE_2D, texture);
    //     gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    //     gl::TexImage2D(
    //         gl::TEXTURE_2D,
    //         0, 
    //         gl::RGBA32F as i32, 
    //         rust_image.width() as i32, 
    //         rust_image.height() as i32, 
    //         0,
    //         gl::RGBA,
    //         gl::UNSIGNED_BYTE,
    //         rust_image.into_raw().as_ptr() as *const std::ffi::c_void
    //     );
    //     gl::GenerateMipmap(gl::TEXTURE_2D);
    // }

    // TODO: this is just test code to make compute shader work
    // dimensions of the image
    let (tex_w, tex_h) = (window_x, window_y);
    let mut tex_output: gl::types::GLuint = 0;
    unsafe { 
        gl::GenTextures(1, &mut tex_output);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, tex_output);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA32F  as i32, tex_w as i32, tex_h as i32, 0, gl::RGBA, gl::FLOAT,std::ptr::null());
        gl::BindImageTexture(0, tex_output, 0, gl::FALSE, 0, gl::READ_WRITE, gl::RGBA32F);
    }
    let tex_output = tex_output;

    let mut vao: gl::types::GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, v_vbo);

        let stride = (5 * std::mem::size_of::<f32>()) as gl::types::GLint;
        gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        gl::VertexAttribPointer(
            0, // index of the generic vertex attribute ("layout (location = 0)")
            3, // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            stride, // stride (byte offset between consecutive attributes)
            std::ptr::null() // offset of the first component
        );

        gl::EnableVertexAttribArray(1); 
        gl::VertexAttribPointer(
            1, 
            2, 
            gl::FLOAT,
            gl::FALSE, 
            stride, 
            (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
        );

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    // Retrieve work group count limit
    let mut work_group_count_limit = [0, 0, 0];
    unsafe {
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_COUNT, 0, &mut work_group_count_limit[0]);
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_COUNT, 1, &mut work_group_count_limit[1]);
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_COUNT, 2, &mut work_group_count_limit[2]);
    }
    let work_group_count_limit = work_group_count_limit;

    // Retrieve work group size limit
    let mut work_group_size_limit = [0, 0, 0];
    unsafe {
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_SIZE, 0, &mut work_group_size_limit[0]);
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_SIZE, 1, &mut work_group_size_limit[1]);
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_SIZE, 2, &mut work_group_size_limit[2]);
    }
    let work_group_size_limit = work_group_size_limit;

    let mut work_group_invocation_limit = 0;
    unsafe {
        gl::GetIntegerv(gl::MAX_COMPUTE_WORK_GROUP_INVOCATIONS, &mut work_group_invocation_limit);
    }
    let work_group_invocation_limit = work_group_invocation_limit;

    let state_update_comp = {
        let shader = renderer::shader::Shader::from_resources(&res, "shaders/state_update.comp").unwrap();
        Program::from_shaders(&[shader]).unwrap()
    }; 


    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode, .. } => println!("Keydown: {:?}", keycode),
                Event::KeyUp { keycode, .. } => println!("Keyup: {:?}", keycode),
                _ => {}
            }
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        state_update_comp.set_used();
        unsafe {
            gl::DispatchCompute(tex_w, tex_h, 1);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        }

        triangle_program.set_used();

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, tex_output);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, i_vbo);

            gl::DrawElements(
                gl::TRIANGLES, 
                indices.len() as i32, 
                gl::UNSIGNED_INT,
                std::ptr::null()
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        window.gl_swap_window();
    }

}