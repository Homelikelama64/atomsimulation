use std::collections::HashMap;

use cgmath::{prelude::*, Vector2, Vector3};
use eframe::{
    egui,
    egui_wgpu::{Callback, WgpuConfiguration},
    wgpu::{self},
    NativeOptions, Renderer,
};
use physics::{update_particles, Bond, Element, Particle, Rectangle};
use rendering::{create_render_state, GpuCamera, GpuCircle, GpuRectangle, RenderCallback};

mod physics;
mod rendering;

struct Camera {
    position: Vector2<f32>,
    zoom: f32,
}

enum SelectedObject {
    Particle(usize),
    Rectangle(usize),
}

struct App {
    last_frame_time: Option<std::time::Instant>,
    info_window_open: bool,
    selected_object: Option<SelectedObject>,
    time_scale: usize,
    camera: Camera,
    particles: Vec<Particle>,
    bonds: HashMap<(usize, usize), Bond>,
    rectangles: Vec<Rectangle>,
}

impl App {
    fn new(cc: &eframe::CreationContext) -> App {
        create_render_state(cc);
        App {
            last_frame_time: None,
            info_window_open: true,
            selected_object: None,
            time_scale: 1,
            camera: Camera {
                position: Vector2 { x: 0.0, y: 0.0 },
                zoom: 0.25,
            },
            particles: vec![
                Particle {
                    position: Vector2 { x: 3.0, y: 0.0 },
                    velocity: Vector2 { x: -1.0, y: 0.0 },
                    element: Element::Oxygen,
                },
                Particle {
                    position: Vector2 { x: -3.0, y: 0.0 },
                    velocity: Vector2 { x: 0.0, y: 0.0 },
                    element: Element::Hydrogen,
                },
                Particle {
                    position: Vector2 { x: -6.0, y: 0.5 },
                    velocity: Vector2 { x: 30.0, y: 10.0 },
                    element: Element::Hydrogen,
                },
            ],
            bonds: HashMap::from([((0, 1), Bond {}), ((0, 2), Bond {})]),
            rectangles: vec![
                Rectangle {
                    position: Vector2 { x: -15.0, y: 0.0 },
                    color: Vector3 {
                        x: 0.1,
                        y: 0.1,
                        z: 0.1,
                    },
                    size: Vector2 { x: 1.0, y: 16.0 },
                },
                Rectangle {
                    position: Vector2 { x: 15.0, y: 0.0 },
                    color: Vector3 {
                        x: 0.1,
                        y: 0.1,
                        z: 0.1,
                    },
                    size: Vector2 { x: 1.0, y: 16.0 },
                },
                Rectangle {
                    position: Vector2 { x: 0.0, y: 7.5 },
                    color: Vector3 {
                        x: 0.1,
                        y: 0.1,
                        z: 0.1,
                    },
                    size: Vector2 { x: 30.0, y: 1.0 },
                },
                Rectangle {
                    position: Vector2 { x: 0.0, y: -7.5 },
                    color: Vector3 {
                        x: 0.1,
                        y: 0.1,
                        z: 0.1,
                    },
                    size: Vector2 { x: 30.0, y: 1.0 },
                },
            ],
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let time = std::time::Instant::now();
        let dt = time
            .duration_since(self.last_frame_time.unwrap_or(time))
            .as_secs_f32();
        self.last_frame_time = Some(time);

        for _ in 0..self.time_scale {
            update_particles(
                &mut self.particles,
                &mut self.bonds,
                &mut self.rectangles,
                dt,
            );
        }

        egui::TopBottomPanel::top("Menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.info_window_open |= ui.button("Info").clicked();
            });
        });

        egui::Window::new("Info")
            .open(&mut self.info_window_open)
            .show(ctx, |ui| {
                ui.label(format!("FPS: {:.3}", 1.0 / dt));
                ui.label(format!("Frame Time: {:.3}ms", 1000.0 * dt));

                // TODO: make this more accurate
                // let mut energy = 0.0;
                // for particle in &self.particles {
                //     energy += 0.5 * particle.mass() * particle.velocity.magnitude2();
                // }
                // for bond in &self.bonds {
                //     let distance = self.particles[bond.particle_a]
                //         .position
                //         .distance(self.particles[bond.particle_b].position)
                //         - (self.particles[bond.particle_a].radius()
                //             + self.particles[bond.particle_b].radius());
                //     energy += 0.5 * bond.strength(&self.particles) * (distance * distance);
                // }
                // ui.label(format!("Energy: {:.3}", energy));

                ui.horizontal(|ui| {
                    ui.label("Time Scale: ");
                    ui.add(egui::Slider::new(&mut self.time_scale, 0..=20));
                });

                ui.allocate_space(ui.available_size());
            });

        let mut selected_object_window_open = self.selected_object.is_some();
        egui::Window::new("Selected Object")
            .open(&mut selected_object_window_open)
            .show(ctx, |ui| {
                match self.selected_object {
                    Some(SelectedObject::Particle(i)) => {
                        ui.label("Particle:");
                        ui.horizontal(|ui| {
                            ui.label("Position:");
                            ui.add(
                                egui::DragValue::new(&mut self.particles[i].position.x)
                                    .prefix("x:"),
                            );
                            ui.add(
                                egui::DragValue::new(&mut self.particles[i].position.y)
                                    .prefix("y:"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Velocity:");
                            ui.add(
                                egui::DragValue::new(&mut self.particles[i].velocity.x)
                                    .prefix("x:"),
                            );
                            ui.add(
                                egui::DragValue::new(&mut self.particles[i].velocity.y)
                                    .prefix("y:"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Radius:");
                            ui.add(egui::DragValue::new(&mut self.particles[i].radius()));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Mass:");
                            ui.add(egui::DragValue::new(&mut self.particles[i].mass()));
                        });
                        ui.add_enabled_ui(false, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Kinetic Energy:");
                                ui.add(egui::DragValue::new(
                                    &mut (0.5
                                        * self.particles[i].mass()
                                        * self.particles[i].velocity.magnitude2()),
                                ));
                            });
                        });
                        ui.label(format!(
                            "Element: {}",
                            match self.particles[i].element {
                                Element::Hydrogen => "Hydrogen",
                                Element::Oxygen => "Oxygen",
                            },
                        ));
                    }
                    Some(SelectedObject::Rectangle(i)) => {
                        ui.label("Rectangle:");
                        ui.horizontal(|ui| {
                            ui.label("Position:");
                            ui.add(
                                egui::DragValue::new(&mut self.rectangles[i].position.x)
                                    .prefix("x:"),
                            );
                            ui.add(
                                egui::DragValue::new(&mut self.rectangles[i].position.y)
                                    .prefix("y:"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            egui::color_picker::color_edit_button_rgb(
                                ui,
                                self.rectangles[i].color.as_mut(),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Size:");
                            ui.add(
                                egui::DragValue::new(&mut self.rectangles[i].size.x)
                                    .prefix("width:"),
                            );
                            ui.add(
                                egui::DragValue::new(&mut self.rectangles[i].size.y)
                                    .prefix("height:"),
                            );
                        });
                    }
                    None => unreachable!(),
                }
                ui.allocate_space(ui.available_size());
            });
        if !selected_object_window_open {
            self.selected_object = None;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(0, 0, 0)))
            .show(ctx, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
                let aspect = rect.width() / rect.height();

                if response.dragged_by(egui::PointerButton::Secondary) {
                    let delta = response.drag_delta();
                    self.camera.position.x -=
                        delta.x / self.camera.zoom / rect.width() * 2.0 * aspect;
                    self.camera.position.y += delta.y / self.camera.zoom / rect.height() * 2.0;
                }

                if response.clicked_by(egui::PointerButton::Primary) {
                    let mouse_position = ((response.interact_pointer_pos().unwrap()
                        - rect.left_top())
                        / rect.size()
                        * 2.0
                        - egui::vec2(1.0, 1.0))
                        * egui::vec2(1.0, -1.0);
                    let world_position = Vector2 {
                        x: mouse_position.x * aspect / self.camera.zoom + self.camera.position.x,
                        y: mouse_position.y / self.camera.zoom + self.camera.position.y,
                    };
                    self.selected_object = 'search: {
                        for (i, particle) in self.particles.iter().enumerate() {
                            if (world_position - particle.position).magnitude2()
                                <= particle.radius() * particle.radius()
                            {
                                break 'search Some(SelectedObject::Particle(i));
                            }
                        }
                        for (i, rectangle) in self.rectangles.iter().enumerate() {
                            let relative_position = world_position - rectangle.position;
                            if relative_position.x.abs() <= rectangle.size.x * 0.5
                                && relative_position.y.abs() <= rectangle.size.y * 0.5
                            {
                                break 'search Some(SelectedObject::Rectangle(i));
                            }
                        }
                        None
                    };
                }

                if response.hovered() {
                    ctx.input(|input| match input.scroll_delta.y.total_cmp(&0.0) {
                        std::cmp::Ordering::Less => self.camera.zoom *= 0.9,
                        std::cmp::Ordering::Greater => self.camera.zoom /= 0.9,
                        _ => {}
                    });
                }

                ui.painter().add(Callback::new_paint_callback(
                    rect,
                    RenderCallback {
                        camera: GpuCamera {
                            position: self.camera.position,
                            aspect,
                            zoom: self.camera.zoom,
                        },
                        circles: self
                            .particles
                            .iter()
                            .map(|particle| GpuCircle {
                                position: particle.position,
                                color: particle.color(),
                                radius: particle.radius(),
                            })
                            .collect(),
                        rectangles: self
                            .rectangles
                            .iter()
                            .map(|rectangle| GpuRectangle {
                                position: rectangle.position,
                                color: rectangle.color,
                                size: rectangle.size,
                            })
                            .collect(),
                    },
                ));
            });

        ctx.request_repaint();
    }
}

fn main() {
    eframe::run_native(
        "Rocket Simulator",
        NativeOptions {
            vsync: false,
            renderer: Renderer::Wgpu,
            wgpu_options: WgpuConfiguration {
                power_preference: wgpu::PowerPreference::HighPerformance,
                present_mode: wgpu::PresentMode::AutoNoVsync,
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(|cc| Box::new(App::new(cc))),
    )
    .unwrap();
}
