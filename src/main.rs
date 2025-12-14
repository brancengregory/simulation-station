use eframe::egui;

pub trait Simulation {
    fn name(&self) -> &str;
    fn update(&mut self);
    fn render(&self, buffer: &mut Vec<u8>);
    fn reset(&mut self);
    fn ui(&mut self, ui: &mut egui::Ui);
}

pub struct NoSim;
impl Simulation for NoSim {
    fn name(&self) -> &str { "None" }
    fn update(&mut self) {}
    fn reset(&mut self) {}
    fn ui(&mut self, ui: &mut egui::Ui) { ui.label("No simulation selected."); }
    fn render(&self, buffer: &mut Vec<u8>) { buffer.fill(0); }
}

struct App {
    current_sim: Box<dyn Simulation>,
    is_paused: bool,
    speed: usize,
    texture: Option<egui::TextureHandle>,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_sim: Box::new(NoSim),
            is_paused: false,
            speed: 1,
            texture: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("controls").min_width(200.0).show(ctx, |ui| {
            ui.heading("Simulation Station");
            ui.separator();

            ui.label("Load Simulation:");
            egui::ComboBox::from_id_salt("sim_select")
                .selected_text(self.current_sim.name())
                .show_ui(ui, |ui| {
                    if ui.selectable_label(false, "None").clicked() {
                        self.current_sim = Box::new(NoSim);
                    }
                });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button(if self.is_paused { "Resume" } else { "Pause" }).clicked() {
                    self.is_paused = !self.is_paused;
                }
                if ui.button("Reset").clicked() {
                    self.current_sim.reset();
                }
            });

            ui.add(egui::Slider::new(&mut self.speed, 1..=100).text("Speed"));

            ui.separator();

            self.current_sim.ui(ui);
        });

        if !self.is_paused {
            for _ in 0..self.speed {
                self.current_sim.update();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let w = 400;
            let h = 300;
            let mut pixel_buffer = Vec::with_capacity(w * h * 3);

            self.current_sim.render(&mut pixel_buffer);

            if pixel_buffer.len() != w * h * 3 {
                pixel_buffer.resize(w * h * 3, 0);
            }

            let image = egui::ColorImage::from_rgb([w, h], &pixel_buffer);
            self.texture = Some(ctx.load_texture("display", image, egui::TextureOptions::NEAREST));

            if let Some(texture) = &self.texture {
                ui.image((texture.id(), ui.available_size()));
            }
        });

        ctx.request_repaint();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Simulation Station",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast;
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("main").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(App::new(cc)))),
            )
            .await
            .expect("failed to start eframe");
    });
}
