use std::collections::HashMap;

use cgmath::{prelude::*, Vector2, Vector3};

#[derive(Clone, Copy)]
pub enum Element {
    Hydrogen,
    Oxygen,
}

impl Element {
    pub fn color(&self) -> Vector3<f32> {
        match self {
            Self::Hydrogen => Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Self::Oxygen => Vector3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        }
    }

    pub fn mass(&self) -> f32 {
        match self {
            Self::Hydrogen => 1.0,
            Self::Oxygen => 16.0,
        }
    }
}

pub struct Particle {
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub element: Element,
}

impl Particle {
    pub fn color(&self) -> Vector3<f32> {
        self.element.color()
    }

    pub fn radius(&self) -> f32 {
        (self.mass() / std::f32::consts::PI).sqrt()
    }

    pub fn mass(&self) -> f32 {
        self.element.mass()
    }
}

pub struct Rectangle {
    pub position: Vector2<f32>,
    pub color: Vector3<f32>,
    pub size: Vector2<f32>,
}

pub struct Bond {}

impl Bond {
    pub const FORCE: f32 = 1.0;
    pub const DISTANCE: f32 = 1.0;

    pub fn strength(a: &Particle, b: &Particle) -> f32 {
        match (b.element, a.element) {
            (Element::Hydrogen, Element::Hydrogen) => 4.36,
            (Element::Hydrogen, Element::Oxygen) => 4.59,
            (Element::Oxygen, Element::Hydrogen) => 4.59,
            (Element::Oxygen, Element::Oxygen) => 1.42, // TODO: what about double bonds????
        }
    }
}

pub fn update_particles(
    particles: &mut Vec<Particle>,
    bonds: &mut HashMap<(usize, usize), Bond>,
    rectangles: &mut [Rectangle],
    dt: f32,
) {
    const MAX_ITERATIONS: usize = 100;

    let mut reached_max_iterations = true;
    for _ in 0..MAX_ITERATIONS {
        let mut was_collision = false;

        for i in 0..particles.len() {
            for j in i + 1..particles.len() {
                let distance = particles[i].position.distance(particles[j].position);
                if distance < particles[i].radius() + particles[j].radius() {
                    let relvel = particles[i].velocity - particles[j].velocity;
                    let dir = (particles[i].position - particles[j].position) / distance;
                    if relvel.dot(dir) < 0.0 {
                        was_collision = true;

                        let m1 = particles[i].mass();
                        let m2 = particles[j].mass();
                        let v1 = particles[i].velocity;
                        let v2 = particles[j].velocity;
                        let x1 = particles[i].position;
                        let x2 = particles[j].position;

                        // https://en.wikipedia.org/wiki/Elastic_collision#Two-dimensional_collision_with_two_moving_objects
                        particles[i].velocity = v1
                            - (x1 - x2)
                                * ((2.0 * m2) / (m1 + m2))
                                * ((v1 - v2).dot(x1 - x2) / (distance * distance));

                        particles[j].velocity = v2
                            - (x2 - x1)
                                * ((2.0 * m1) / (m1 + m2))
                                * ((v2 - v1).dot(x2 - x1) / (distance * distance));
                    }
                }
            }

            let particle = &mut particles[i];
            for rectangle in &*rectangles {
                let relative_particle_position = particle.position - rectangle.position;
                let mut closest_point = relative_particle_position;
                closest_point.x = closest_point
                    .x
                    .clamp(-rectangle.size.x * 0.5, rectangle.size.x * 0.5);
                closest_point.y = closest_point
                    .y
                    .clamp(-rectangle.size.y * 0.5, rectangle.size.y * 0.5);
                if closest_point.distance2(relative_particle_position)
                    < particle.radius() * particle.radius()
                {
                    let normal = (closest_point - relative_particle_position).normalize();
                    if normal.dot(particle.velocity) > 0.0 {
                        was_collision = true;

                        particle.velocity =
                            particle.velocity - 2.0 * particle.velocity.dot(normal) * normal;
                    }
                }
            }
        }

        if !was_collision {
            reached_max_iterations = false;
            break;
        }
    }
    if reached_max_iterations {
        println!("WARNING: Max iterations reached, the simulation may be unstable");
    }

    bonds.retain(|&(a, b), _bond| {
        let distance = particles[a].position.distance(particles[b].position)
            - (particles[a].radius() + particles[b].radius() + Bond::DISTANCE);
        let a_to_b = particles[b].position - particles[a].position;
        let force = Bond::FORCE * distance;
        if force > Bond::strength(&particles[a], &particles[b]) {
            return false;
        }
        let a_mass = particles[a].mass();
        let b_mass = particles[b].mass();
        particles[a].velocity += a_to_b * force * ((2.0 * b_mass) / (a_mass + b_mass)) * dt;
        particles[b].velocity -= a_to_b * force * ((2.0 * a_mass) / (a_mass + b_mass)) * dt;
        true
    });

    for particle in particles {
        particle.position += particle.velocity * dt;
    }
}
