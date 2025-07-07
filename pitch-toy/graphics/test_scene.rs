use web_sys::WebGl2RenderingContext;

pub struct TestScene {
    shader_program: Option<web_sys::WebGlProgram>,
    vertex_buffer: Option<web_sys::WebGlBuffer>,
    vertex_array: Option<web_sys::WebGlVertexArrayObject>,
}

impl TestScene {
    pub fn new() -> Self {
        Self {
            shader_program: None,
            vertex_buffer: None,
            vertex_array: None,
        }
    }

    pub fn initialize(&mut self, gl: &WebGl2RenderingContext) -> Result<(), String> {
        // Create basic 2D shader program and geometry for test rectangle
        self.create_basic_shader_program(gl)?;
        self.create_rectangle_geometry(gl)?;
        
        web_sys::console::log_1(&"✓ Test scene initialized with visible green rectangle".into());
        
        Ok(())
    }

    pub fn render(&self, gl: &WebGl2RenderingContext) -> Result<(), String> {
        // Render the test rectangle
        if let (Some(program), Some(vao)) = (&self.shader_program, &self.vertex_array) {
            // Use our shader program
            gl.use_program(Some(program));
            
            // Bind vertex array object
            gl.bind_vertex_array(Some(vao));
            
            // Draw the rectangle (2 triangles = 6 vertices)
            gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
            
            // Cleanup
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
        
        Ok(())
    }

    fn create_basic_shader_program(&mut self, gl: &WebGl2RenderingContext) -> Result<(), String> {
        // Create basic vertex shader for 2D rendering
        let vertex_shader_source = r#"#version 300 es
            in vec2 a_position;
            
            void main() {
                gl_Position = vec4(a_position, 0.0, 1.0);
            }
        "#;
        
        // Create basic fragment shader - bright green color to be clearly visible
        let fragment_shader_source = r#"#version 300 es
            precision mediump float;
            out vec4 fragColor;
            
            void main() {
                // Bright green color to demonstrate 2D rendering works
                fragColor = vec4(0.0, 1.0, 0.0, 1.0);
            }
        "#;
        
        // Compile shaders
        let vertex_shader = self.compile_shader(gl, WebGl2RenderingContext::VERTEX_SHADER, vertex_shader_source)?;
        let fragment_shader = self.compile_shader(gl, WebGl2RenderingContext::FRAGMENT_SHADER, fragment_shader_source)?;
        
        // Create and link program
        let program = gl.create_program().ok_or("Failed to create shader program")?;
        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);
        
        // Check if program linked successfully
        if !gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS).as_bool().unwrap_or(false) {
            let error = gl.get_program_info_log(&program).unwrap_or_default();
            return Err(format!("Failed to link shader program: {}", error));
        }
        
        // Clean up shaders
        gl.delete_shader(Some(&vertex_shader));
        gl.delete_shader(Some(&fragment_shader));
        
        self.shader_program = Some(program);
        
        web_sys::console::log_1(&"✓ Test scene shader program compiled and linked".into());
        
        Ok(())
    }
    
    fn compile_shader(&self, gl: &WebGl2RenderingContext, shader_type: u32, source: &str) -> Result<web_sys::WebGlShader, String> {
        let shader = gl.create_shader(shader_type).ok_or("Failed to create shader")?;
        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);
        
        if !gl.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false) {
            let error = gl.get_shader_info_log(&shader).unwrap_or_default();
            return Err(format!("Failed to compile shader: {}", error));
        }
        
        Ok(shader)
    }
    
    fn create_rectangle_geometry(&mut self, gl: &WebGl2RenderingContext) -> Result<(), String> {
        // Create a simple rectangle geometry (two triangles)
        // Centered rectangle from -0.3 to 0.3 in both dimensions (clearly visible)
        #[rustfmt::skip]
        let vertices: [f32; 12] = [
            // First triangle
            -0.3, -0.3,  // Bottom left
             0.3, -0.3,  // Bottom right
            -0.3,  0.3,  // Top left
            
            // Second triangle
            -0.3,  0.3,  // Top left
             0.3, -0.3,  // Bottom right
             0.3,  0.3,  // Top right
        ];
        
        // Create vertex buffer
        let buffer = gl.create_buffer().ok_or("Failed to create vertex buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
        
        // Upload vertex data
        unsafe {
            let vertex_array = js_sys::Float32Array::view(&vertices);
            gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vertex_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
        
        // Create vertex array object
        let vao = gl.create_vertex_array().ok_or("Failed to create vertex array")?;
        gl.bind_vertex_array(Some(&vao));
        
        // Set up vertex attributes
        if let Some(program) = &self.shader_program {
            let position_attrib = gl.get_attrib_location(program, "a_position") as u32;
            gl.enable_vertex_attrib_array(position_attrib);
            gl.vertex_attrib_pointer_with_i32(
                position_attrib,
                2,  // 2 components per vertex (x, y)
                WebGl2RenderingContext::FLOAT,
                false,
                0,  // stride
                0,  // offset
            );
        }
        
        // Unbind
        gl.bind_vertex_array(None);
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        
        self.vertex_buffer = Some(buffer);
        self.vertex_array = Some(vao);
        
        web_sys::console::log_1(&"✓ Test rectangle geometry created with 6 vertices".into());
        
        Ok(())
    }
}

impl Default for TestScene {
    fn default() -> Self {
        Self::new()
    }
}