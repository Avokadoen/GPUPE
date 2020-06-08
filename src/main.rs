extern crate gl;
extern crate sdl2;
extern crate image;
extern crate cgmath;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::path::Path;
use std::time::Instant;

mod renderer;
mod utility;
mod resources;

use resources::Resources;
use renderer::{
    program::Program,
    texture::Texture, 
    vao::{
        VertexArrayObject,
        VertexAttributePointer
    }
};

use utility::{
    camera::Camera2D, 
    input_handler::InputHandler,
    direction::Direction,
};

// TODO: currently lots of opengl stuff. Move all of it into renderer module

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

    // NOTE: CHUNK SIZE
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
    let mut triangle_program = Program::from_resources(&res, "shaders/triangle").unwrap();

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

    // TODO: 3D texture 

    // // TODO: this is just test code to make compute shader work, we need abstractions to make this prettier and more generic
    // // dimensions of the image
    let image = res.load_image("textures/water_test.png")
        .unwrap()
        .into_rgba();
    let state_output = Texture::from_image(image, gl::TEXTURE0, 0, gl::RGBA32F, gl::RGBA);
    let updated_map = Texture::new(gl::TEXTURE1, 1, gl::R8, gl::RED);
    let velocity_map = Texture::new(gl::TEXTURE2, 2, gl::RG32F, gl::RG);

    let vao = { 
        let pos = VertexAttributePointer {
            location: 0,
            size: 3,
            offset: 0
        };

        let uv = VertexAttributePointer {
            location: 1,
            size: 2,
            offset: 3
        };

        VertexArrayObject::new(vec![pos, uv], 5, v_vbo)
    };
    // let mut vao: gl::types::GLuint = 0;
    // unsafe {
    //     gl::GenVertexArrays(1, &mut vao);

    //     gl::BindVertexArray(vao);
    //     gl::BindBuffer(gl::ARRAY_BUFFER, v_vbo);

    //     let stride = (5 * std::mem::size_of::<f32>()) as gl::types::GLint;
    //     gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
    //     gl::VertexAttribPointer(
    //         0,                  // index of the generic vertex attribute ("layout (location = 0)")
    //         3,                  // the number of components per generic vertex attribute
    //         gl::FLOAT,          // data type
    //         gl::FALSE,          // normalized (int-to-float conversion)
    //         stride,             // stride (byte offset between consecutive attributes)
    //         std::ptr::null()    // offset of the first component
    //     );

    //     gl::EnableVertexAttribArray(1); 
    //     gl::VertexAttribPointer(
    //         1, 
    //         2, 
    //         gl::FLOAT,
    //         gl::FALSE, 
    //         stride, 
    //         (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
    //     );

    //     gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    //     gl::BindVertexArray(0);
    // }

    // TODO: create a compute shader abstraction, used this in the abstraction somewhere where it can be shared
    // Retrieve work group count limit
    let mut work_group_count_limit = [0, 0, 0];
    unsafe {
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_COUNT, 0, &mut work_group_count_limit[0]);
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_COUNT, 1, &mut work_group_count_limit[1]);
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_COUNT, 2, &mut work_group_count_limit[2]);
    }
    let _work_group_count_limit = work_group_count_limit;

    // Retrieve work group size limit
    let mut work_group_size_limit = [0, 0, 0];
    unsafe {
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_SIZE, 0, &mut work_group_size_limit[0]);
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_SIZE, 1, &mut work_group_size_limit[1]);
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_SIZE, 2, &mut work_group_size_limit[2]);
    }
    let _work_group_size_limit = work_group_size_limit;

    let mut work_group_invocation_limit = 0;
    unsafe {
        gl::GetIntegerv(gl::MAX_COMPUTE_WORK_GROUP_INVOCATIONS, &mut work_group_invocation_limit);
    }
    let _work_group_invocation_limit = work_group_invocation_limit;

    let mut state_update_comp = {
        let shader = renderer::shader::Shader::from_resources(&res, "shaders/state_update.comp").unwrap();
        Program::from_shaders(&[shader]).unwrap()
    }; 

    fn dispatch_compute(state_update_comp: &mut Program) {
        state_update_comp.set_used();

        // TODO: we don't really need to loop and dispatch. We can do all passes in one dispatch! (execpt cleanup)
        for pass_type in (0..4).rev() {
            // TODO: don't unwrap
            state_update_comp.set_i32("pass_type", pass_type).unwrap();
            
            // NOTE: CHUNK SIZE
            // TODO: this should not be hardcoded. Should be handled by some compute state abstraction
            // 512 / 8 = 64
            let chunk_size = {
                if pass_type < 3 {
                    64
                } else {
                    512
                }
            };

            unsafe {
                gl::DispatchCompute(chunk_size, chunk_size, 1);
                gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
            }
        }
    }

    // We only use these two textures, so we bind them before render loop and forget about them
    // this is somewhat bad practice, but in our case the consequenses are non existant
    state_output.bind();
    updated_map.bind();
    velocity_map.bind();

    // TODO: put in utility
    let mut now = Instant::now();
    let mut last: Instant;
    let mut delta_time: f64;

    // TODO: put in utlity
    let mut second_tick: f64 = 0.0;
    let mut frames: i32 = 0;

    let mut camera: Camera2D = Default::default();
    let mut input_handler: InputHandler = Default::default();

    // TODO: lock screen from being stretched
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        last = now;
        now = Instant::now();
        delta_time = (last.elapsed().as_millis() - now.elapsed().as_millis()) as f64 / 1000.0;
        second_tick += delta_time;
        frames += 1;
        if second_tick > 1.0 {
            second_tick = 0.0;
            println!("fps: {}", frames);
            frames = 0;
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode, .. } => input_handler.on_key_down(keycode),
                Event::KeyUp { keycode, .. } => input_handler.on_key_up(keycode),
                Event::MouseWheel { y, ..} => {
                    camera.modify_zoom(delta_time, y as f32);
                }, 
                _ => {}
            }
        }

        // TODO: this should be done by some sort of observer like pattern, but this will work for now
        //       as soon as we need runtime config for keybindings this will be a problem
        for keycode in &input_handler.active_keys {
            match keycode {
                Keycode::W => camera.pan_in_direction(Direction::Up),
                Keycode::A => camera.pan_in_direction(Direction::Left),
                Keycode::S => camera.pan_in_direction(Direction::Down),
                Keycode::D => camera.pan_in_direction(Direction::Rigth),
                _ => ()
            }
        }

        if camera.commit_pan_zoom(delta_time) {
            // TODO: error handling for this
            match triangle_program.set_vector3_f32("cameraPos", camera.position()) {
                Ok(()) => (),
                Err(err) => println!("got error setting cameraPos: {}", err)
            }
        }


        dispatch_compute(&mut state_update_comp);
        triangle_program.set_used();
        vao.bind();

        unsafe {
            // TODO: keybinding to select desired texture at runtime instead ..
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, i_vbo);

            gl::DrawElements(
                gl::TRIANGLES, 
                indices.len() as i32, 
                gl::UNSIGNED_INT,
                std::ptr::null()
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        vao.unbind();
        window.gl_swap_window();
    }
    // texture delete ...
}