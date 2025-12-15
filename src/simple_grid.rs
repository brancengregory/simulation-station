use eframe::egui;
use crate::Simulation;

#[derive(Clone)]
pub struct Grid<T> {
    width: usize,
    height: usize,
    cells: Vec<T>,
}

impl<T: Clone + Default> Grid<T> {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![T::default(); width * height],
        }
    }
}

pub struct PixelFillSim {
    grid: Grid<u8>,
    cursor_idx: usize,
}

impl PixelFillSim {
    pub fn new() -> Self {
        let mut sim = Self {
            grid: Grid::new(400, 300),
            cursor_idx: 0,
        };
        sim.reset();
        sim
    }
}

impl Simulation for PixelFillSim {
    fn name(&self) -> &str {
        "Simple Pixel Fill"
    }

    fn reset(&mut self) {
        self.grid.cells.fill(0);
        self.cursor_idx = 0;
    }

    fn update(&mut self) {
        if self.cursor_idx < self.grid.cells.len() {
            self.grid.cells[self.cursor_idx] = 255;
            self.cursor_idx += 1;
        }
    }

    fn render(&self, buffer: &mut Vec<u8>) {
        buffer.clear();
        for &val in &self.grid.cells {
            if val > 0 {
                buffer.extend_from_slice(&[0, 255, 255]);
            } else {
                buffer.extend_from_slice(&[20, 20, 20]);
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Pixels Filled: {}", self.cursor_idx));
        ui.label(format!("Total: {}", self.grid.cells.len()));

        if ui.button("Fill 1000x").clicked() {
            for _ in 0..1000 {
                self.update();
            }
        }
    }
}

