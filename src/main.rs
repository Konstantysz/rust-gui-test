use std::sync::{Arc, Mutex};

use eframe::egui;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let app_name: &str = "Custom 3D painting in eframe using glow";
    let native_options: eframe::NativeOptions = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(350.0, 380.0)),
        multisampling: 4,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    let app_creator: eframe::AppCreator = Box::new(|cc| Box::new(MyApp::new(cc)));
    eframe::run_native(app_name, native_options, app_creator)
}
struct MyApp {
    rotatin_triangle: Arc<Mutex<RotatingTriangle>>,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc
            .gl
            .as_ref()
            .expect("You need to run eframe with glow backend");
        Self {
            rotatin_triangle: Arc::new(Mutex::new(RotatingTriangle::new(gl))),
        }
    }

    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, _response) =
            ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::hover());

        let rotating_triangle = self.rotatin_triangle.clone();

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                rotating_triangle
                    .lock()
                    .expect("Cannot lock mutex to paint triangle.")
                    .paint(painter.gl());
            })),
        };
        ui.painter().add(callback);
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("The triangle is being painted using ");
                ui.hyperlink_to("glow", "https://github.com/grovesNL/glow");
                ui.label(" (OpenGL).")
            });

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.rotatin_triangle
                .lock()
                .expect("Cannot lock mutex to destroy triangle.")
                .destroy(gl);
        }
    }
}

struct RotatingTriangle {
    program: glow::Program,
    vertex_array_object: glow::NativeVertexArray,
    vertex_buffer_object: glow::NativeBuffer,
    index_buffer_object: glow::NativeBuffer,
    counter: f32,
}

impl RotatingTriangle {
    fn new(gl: &glow::Context) -> Self {
        use glow::HasContext as _;

        unsafe {
            let program = create_program(&gl);

            let vertex_buffer_object = gl.create_buffer().expect("Cannot create vertex buffer.");

            let vertex_array_object = gl
                .create_vertex_array()
                .expect("Cannot create vertex array.");

            let index_buffer_object = gl.create_buffer().expect("Cannot create index buffer.");

            Self {
                program,
                vertex_array_object,
                vertex_buffer_object,
                index_buffer_object,
                counter: 0.0f32,
            }
        }
    }

    fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array_object);
            gl.delete_buffer(self.vertex_buffer_object);
        }
    }

    fn paint(&mut self, gl: &glow::Context) {
        use glow::HasContext as _;

        let vertices = [
            -0.5f32, -0.5f32, 0.5f32, -0.5f32, 0.5f32, 0.5f32, -0.5f32, 0.5f32,
        ];
        let indices = [0u32, 1u32, 2u32, 2u32, 3u32, 0u32];

        unsafe {
            let vertices_u8: &[u8] = core::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * core::mem::size_of::<f32>(),
            );
            let indices_u8: &[u8] = core::slice::from_raw_parts(
                indices.as_ptr() as *const u8,
                indices.len() * core::mem::size_of::<u32>(),
            );

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer_object));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_u8, glow::STATIC_DRAW);

            gl.bind_vertex_array(Some(self.vertex_array_object));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.index_buffer_object));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices_u8, glow::STATIC_DRAW);

            gl.use_program(Some(self.program));

            let location = gl.get_uniform_location(self.program, "u_color");
            gl.uniform_4_f32(location.as_ref(), self.counter, 0.2, 0.2, 1.0);

            gl.draw_elements(glow::TRIANGLES, indices.len() as i32, glow::UNSIGNED_INT, 0);
            
            if self.counter > 1.0f32 {
                self.counter = 0.0f32;
            }

            self.counter += 0.05f32;
        }
    }
}

unsafe fn create_program(gl: &glow::Context) -> glow::NativeProgram {
    use glow::HasContext as _;

    let program = gl.create_program().expect("Cannot create program.");

    let shader_version = if cfg!(target_arch = "wasm32") {
        "#version 300 es"
    } else {
        "#version 330"
    };

    let vertex_shader = create_shader(
        gl,
        glow::VERTEX_SHADER,
        VERTEX_SHADER_SOURCE,
        shader_version,
    );
    let fragment_shader = create_shader(
        gl,
        glow::FRAGMENT_SHADER,
        FRAGMENT_SHADER_SOURCE,
        shader_version,
    );

    gl.attach_shader(program, vertex_shader);
    gl.attach_shader(program, fragment_shader);

    gl.link_program(program);
    assert!(
        gl.get_program_link_status(program),
        "{}",
        gl.get_program_info_log(program)
    );

    gl.detach_shader(program, vertex_shader);
    gl.detach_shader(program, fragment_shader);

    gl.delete_shader(vertex_shader);
    gl.delete_shader(fragment_shader);

    program
}

unsafe fn create_shader(
    gl: &glow::Context,
    shader_type: u32,
    shader_source: &str,
    shader_version: &str,
) -> glow::NativeShader {
    use glow::HasContext as _;

    let shader = gl
        .create_shader(shader_type)
        .expect("Cannot create shader.");
    gl.shader_source(shader, &format!("{shader_version}\n{shader_source}"));
    gl.compile_shader(shader);
    assert!(
        gl.get_shader_compile_status(shader),
        "Failed to compile {shader_type}: {}",
        gl.get_shader_info_log(shader)
    );

    shader
}

const VERTEX_SHADER_SOURCE: &str = r#"
    layout(location = 0) in vec4 position;

    void main() {
        gl_Position = position;
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    layout(location = 0) out vec4 color;

    uniform vec4 u_color;

    void main() {
        color = u_color;
    }
"#;
