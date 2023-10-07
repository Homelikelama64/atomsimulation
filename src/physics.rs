use cgmath::{prelude::*, Vector2, Vector3};
use enum_map::{Enum, EnumMap};

#[derive(Enum, Clone, Copy)]
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

    pub fn electrons_to_share(&self) -> usize {
        match self {
            Element::Hydrogen => 1,
            Element::Oxygen => 2,
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
    pub base_element: Element,
    pub attached_elements: EnumMap<Element, u8>,
}

impl Particle {
    pub fn color(&self) -> Vector3<f32> {
        let mut color = self.base_element.color();
        let mut element_count = 1;
        for (element, &count) in &self.attached_elements {
            color += element.color() * count as f32;
            element_count += count as usize;
        }
        color / element_count as f32
    }

    pub fn electrons_to_share(&self) -> usize {
        let mut electrons = self.base_element.electrons_to_share();
        for (element, &count) in &self.attached_elements {
            let electrons_to_share = element.electrons_to_share();
            assert!(electrons >= electrons_to_share * count as usize);
            electrons -= electrons_to_share * count as usize;
        }
        electrons
    }

    pub fn radius(&self) -> f32 {
        (self.mass() / std::f32::consts::PI).sqrt()
    }

    pub fn mass(&self) -> f32 {
        self.base_element.mass()
            + self
                .attached_elements
                .iter()
                .map(|(element, &count)| element.mass() * count as f32)
                .sum::<f32>()
    }
}

pub struct Rectangle {
    pub position: Vector2<f32>,
    pub color: Vector3<f32>,
    pub size: Vector2<f32>,
}

pub fn update_particles(particles: &mut Vec<Particle>, rectangles: &mut [Rectangle], dt: f32) {
    const MAX_ITERATIONS: usize = 100;

    let mut reached_max_iterations = true;
    for _ in 0..MAX_ITERATIONS {
        let mut was_collision = false;

        let mut particles_to_delete = vec![];
        for i in 0..particles.len() {
            for j in i + 1..particles.len() {
                let distance = particles[i].position.distance(particles[j].position);
                if distance < particles[i].radius() + particles[j].radius() {
                    let relvel = particles[i].velocity - particles[j].velocity;
                    let dir = (particles[i].position - particles[j].position) / distance;
                    if relvel.dot(dir) < 0.0 {
                        let relative_kinetic_energy =
                            (0.5 * particles[i].velocity * particles[i].mass()
                                - 0.5 * particles[j].velocity * particles[j].mass())
                            .magnitude2()
                                * 2.0;
                        let i_electrons_to_share = particles[i].electrons_to_share();
                        let j_electrons_to_share = particles[j].electrons_to_share();
                        if relative_kinetic_energy > 400.0
                            && i_electrons_to_share != 0
                            && j_electrons_to_share != 0
                        {
                            let baseelectronsi = particles[i].base_element.electrons_to_share();
                            let baseelectronsj = particles[j].base_element.electrons_to_share();
                            let ikenetic =
                                0.5 * particles[i].mass() * particles[i].velocity.magnitude2();
                            let jkenetic =
                                0.5 * particles[j].mass() * particles[j].velocity.magnitude2();
                            let final_vel = f32::sqrt(
                                (ikenetic + jkenetic + 0.0)
                                    / (particles[i].mass() + particles[j].mass())
                                    * 2.0,
                            );
                            if baseelectronsi > baseelectronsj {
                                let i_base = particles[i].base_element;
                                let j_base = particles[j].base_element;
                                particles[j].attached_elements[j_base] += 1;
                                particles[j].base_element = i_base;
                            } else if baseelectronsj >= baseelectronsi {
                                let i_base = particles[i].base_element;
                                particles[j].attached_elements[i_base] += 1;
                            }
                            for (element, count) in particles[i].attached_elements {
                                particles[j].attached_elements[element] += count;
                            }
                            particles_to_delete.push(i);
                            particles[j].velocity = particles[j].velocity.normalize() * final_vel;
                        } else {
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
        for particle_to_delete in particles_to_delete {
            particles.remove(particle_to_delete);
        }

        if !was_collision {
            reached_max_iterations = false;
            break;
        }
    }
    if reached_max_iterations {
        println!("WARNING: Max iterations reached, the simulation may be unstable");
    }

    for particle in particles {
        particle.position += particle.velocity * dt;
    }
}
