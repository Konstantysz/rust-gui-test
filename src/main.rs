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
            ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

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
    vertex_array: glow::VertexArray,
}

impl RotatingTriangle {
    fn new(gl: &glow::Context) -> Self {
        use glow::HasContext as _;

        unsafe {
            let program = create_program(&gl);

            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array.");

            Self {
                program,
                vertex_array,
            }
        }
    }

    fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }

    fn paint(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(glow::TRIANGLES, 0, 3)
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

    let vertex_shader = create_shader(gl, glow::VERTEX_SHADER, VERTEX_SHADER_SOURCE, shader_version);
    let fragment_shader = create_shader(gl, glow::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE, shader_version);

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

unsafe fn create_shader(gl: &glow::Context, shader_type: u32, shader_source: &str, shader_version: &str) -> glow::NativeShader {
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
    const vec2 verts[3] = vec2[3](
        vec2(0.0, 1.0),
        vec2(-1.0, -1.0),
        vec2(1.0, -1.0)
    );
    const vec4 colors[3] = vec4[3](
        vec4(1.0, 0.0, 0.0, 1.0),
        vec4(0.0, 1.0, 0.0, 1.0),
        vec4(0.0, 0.0, 1.0, 1.0)
    );
    out vec4 v_color;
    void main() {
        v_color = colors[gl_VertexID];
        gl_Position = vec4(verts[gl_VertexID], 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    precision mediump float;
    in vec4 v_color;
    out vec4 out_color;
    void main() {
        out_color = v_color;
    }
"#;
