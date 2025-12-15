mod simple_grid;
mod p0014;

use eframe::egui;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;


#[derive(Clone, Copy)]
pub struct SimConfig {
    pub min_speed: f32,
    pub max_speed: f32,
    pub default_speed: f32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            min_speed: 1.0,
            max_speed: 10_000.0,
            default_speed: 60.0,
        }
    }
}

pub trait Simulation {
    fn name(&self) -> &str;
    fn config(&self) -> SimConfig { SimConfig::default() }
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

pub struct AsyncSim<T: Send + 'static + Default> {
    name: String,
    config: SimConfig,
    state: T,
    receiver: Option<Receiver<T>>,
    spawner: Arc<dyn Fn(SyncSender<T>) + Send + Sync>,
    renderer: Box<dyn Fn(&T, &mut Vec<u8>) + Send + Sync>,
    ui_draw: Box<dyn Fn(&T, &mut egui::Ui) + Send + Sync>,
}

impl<T: Send + 'static + Default> AsyncSim<T> {
    pub fn new(
        name: &str,
        config: SimConfig,
        spawner: impl Fn(SyncSender<T>) + Send + Sync + 'static,
        renderer: impl Fn(&T, &mut Vec<u8>) + Send + Sync + 'static,
        ui_draw: impl Fn(&T, &mut egui::Ui) + Send + Sync + 'static,
    ) -> Self {
        let mut sim = Self {
            name: name.to_owned(),
            config,
            state: T::default(),
            receiver: None,
            spawner: Arc::new(spawner),
            renderer: Box::new(renderer),
            ui_draw: Box::new(ui_draw),
        };
        sim.reset();
        sim
    }
}

impl<T: Send + 'static + Default> Simulation for AsyncSim<T> {
    fn name(&self) -> &str { &self.name }

    fn config(&self) -> SimConfig { self.config }

    fn update(&mut self) {
        if let Some(rx) = &self.receiver {
            if let Ok(new_state) = rx.try_recv() {
                self.state = new_state;
            }
        }
    }

    fn reset(&mut self) {
        let (tx, rx) = sync_channel(0);
        self.receiver = Some(rx);
        self.state = T::default();

        let spawner = self.spawner.clone();

        std::thread::spawn(move || {
            (spawner)(tx);
        });
    }

    fn render(&self, buffer: &mut Vec<u8>) {
        (self.renderer)(&self.state, buffer);
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        (self.ui_draw)(&self.state, ui);
    }
}

pub struct App {
    current_sim: Box<dyn Simulation>,
    is_paused: bool,
    updates_per_second: f32,
    time_accumulator: f32,
    texture: Option<egui::TextureHandle>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_sim: Box::new(NoSim),
            is_paused: false,
            updates_per_second: 60.0,
            time_accumulator: 0.0,
            texture: None,
        }
    }

    fn load_sim(&mut self, sim: Box<dyn Simulation>) {
        let cfg = sim.config();
        self.updates_per_second = cfg.default_speed;
        self.current_sim = sim;
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
                        self.load_sim(Box::new(NoSim));
                    }

                    if ui.selectable_label(false, "Simple Pixel Fill").clicked() {
                        self.load_sim(Box::new(simple_grid::PixelFillSim::new()));
                    }

                    if ui.selectable_label(false, "Problem 14: Collatz").clicked() {
                        let sim = AsyncSim::new(
                            "Problem 14: Collatz",
                            SimConfig {
                                min_speed: 1.0,
                                max_speed: 50_000.0,
                                default_speed: 10_000.0,
                            },
                            p0014::solve,
                            p0014::render,
                            p0014::ui,
                        );
                        self.load_sim(Box::new(sim));
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

            ui.add(
                egui::Slider::new(&mut self.updates_per_second, 0.5..=10_000.0)
                    .text("Hz (Ops/Sec)")
                    .logarithmic(true)
            );

            ui.separator();

            self.current_sim.ui(ui);
        });

        if !self.is_paused {
            // 1. Get time passed since last frame (Delta Time)
            let dt = ctx.input(|i| i.stable_dt);
            self.time_accumulator += dt;

            // 2. Calculate how long ONE step should take
            // Example: 10 Hz = 0.1s per step
            let step_duration = 1.0 / self.updates_per_second;

            // 3. "Spend" the accumulated time to run updates
            // If speed is 1000Hz, this loop runs ~16 times per 60Hz frame.
            // If speed is 1Hz, this loop runs once every 60 frames.
            let mut loops = 0;
            while self.time_accumulator >= step_duration && loops < 5000 {
                self.current_sim.update(); // Allows thread to proceed one step
                self.time_accumulator -= step_duration;
                loops += 1;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let w = 400;
            let h = 300;
            let mut pixel_buffer = vec![0; w * h * 3];

            self.current_sim.render(&mut pixel_buffer);

            let image = egui::ColorImage::from_rgb([w, h], &pixel_buffer);
            self.texture = Some(ctx.load_texture("display", image, egui::TextureOptions::NEAREST));

            if let Some(texture) = &self.texture {
                ui.image((texture.id(), ui.available_size()));
            }
        });

        ctx.request_repaint();
    }
}
