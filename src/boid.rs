use super::{HEIGHT, WIDTH};
use kd_tree::{KdPoint, KdTree};
use nannou::prelude::*;
use rand::Rng;
use wgpu::Texture;

pub const NUM_POINTS: usize = 900;
const MAX_FORCE: f32 = 0.03;
const MAX_SPEED: f32 = 2.8;
const SEPARATION_RADIUS: f32 = 25.0;
const ALIGNMENT_RADIUS: f32 = 20.0;
const COHESION_RADIUS: f32 = 20.0;
const BOID_RADIUS: f32 = 8.0;

#[derive(Clone, Debug)]
pub struct Boid {
    position: Point2,
    velocity: Point2,
    acceleration: Point2,
    radius: f32,
}

impl Boid {
    pub fn rand_new() -> Self {
        let mut rng = rand::thread_rng();
        let position = Point2::new(
            rng.gen_range(-(WIDTH as f32 / 2.0)..=(WIDTH as f32 / 2.0)),
            rng.gen_range(-(HEIGHT as f32 / 2.0)..=(HEIGHT as f32 / 2.0)),
        );
        let velocity = Point2::new(
            rng.gen_range(-MAX_SPEED..MAX_SPEED),
            rng.gen_range(-MAX_SPEED..MAX_SPEED),
        );
        let acceleration = Point2::new(0.0, 0.0);
        Self {
            position,
            velocity,
            acceleration,
            radius: BOID_RADIUS,
        }
    }

    pub fn show(&self, draw: &Draw, texture: &Texture) {
        let theta = self.velocity.angle();
        draw.texture(texture)
            .xy(self.position)
            .wh(Point2::new(self.radius, self.radius))
            .rotate(theta);
    }

    fn apply_force(&mut self, force: Point2) {
        self.acceleration += force;
    }

    fn separate(&self, kd_tree: &KdTree<Boid>) -> Point2 {
        let mut steer = Point2::new(0.0, 0.0);
        let mut count = 0;
        let neighbors =
            kd_tree.within_radius(&[self.position.x, self.position.y], SEPARATION_RADIUS);
        for other in neighbors {
            let d = self.position - other.position;
            let distance = d.length();
            if distance > 0.0 && distance < SEPARATION_RADIUS {
                let diff = d / distance;
                steer += diff;
                count += 1;
            }
        }

        if count > 0 {
            steer /= count as f32;
        }

        if steer.length() > 0.0 {
            steer = steer.normalize();
            steer *= MAX_SPEED;
            steer -= self.velocity;
            steer = steer.clamp_length_max(MAX_FORCE);
        }

        steer
    }

    fn align(&self, kd_tree: &KdTree<Boid>) -> Point2 {
        let mut sum = Point2::new(0.0, 0.0);
        let mut count = 0;
        let neighbors =
            kd_tree.within_radius(&[self.position.x, self.position.y], ALIGNMENT_RADIUS);
        for other in neighbors {
            let d = self.position - other.position;
            let distance = d.length();
            if distance > 0.0 && distance < ALIGNMENT_RADIUS as f32 {
                sum += other.velocity;
                count += 1;
            }
        }

        if count > 0 {
            sum /= count as f32;
            sum = sum.normalize();
            sum *= MAX_SPEED;
            let mut steer = sum - self.velocity;
            steer = steer.clamp_length_max(MAX_FORCE);
            steer
        } else {
            Point2::new(0.0, 0.0)
        }
    }

    fn cohesion(&self, kd_tree: &KdTree<Boid>) -> Point2 {
        let mut sum = Point2::new(0.0, 0.0);
        let mut count = 0;
        let neighbors = kd_tree.within_radius(&[self.position.x, self.position.y], COHESION_RADIUS);
        for other in neighbors {
            let d = self.position - other.position;
            let distance = d.length();
            if distance > 0.0 && distance < COHESION_RADIUS as f32 {
                sum += other.position;
                count += 1;
            }
        }

        if count > 0 {
            sum /= count as f32;
            self.seek(sum)
        } else {
            Point2::new(0.0, 0.0)
        }
    }

    fn seek(&self, target: Point2) -> Point2 {
        let desired = target - self.position;
        let mut desired = desired.normalize();
        desired *= MAX_SPEED;
        let mut steer = desired - self.velocity;
        steer = steer.clamp_length_max(MAX_FORCE);
        steer
    }

    fn flock(&mut self, kd_tree: &KdTree<Boid>) {
        let mut separation = self.separate(kd_tree);
        let mut alignment = self.align(kd_tree);
        let mut cohesion = self.cohesion(kd_tree);

        // weight forces
        separation = separation * 2.0;
        alignment = alignment * 1.5;
        cohesion = cohesion * 1.3;

        self.apply_force(separation);
        self.apply_force(alignment);
        self.apply_force(cohesion);
    }

    fn update(&mut self) {
        self.velocity += self.acceleration;
        self.velocity = self.velocity.clamp_length_max(MAX_SPEED);
        self.position += self.velocity;
        self.acceleration *= 0.0;
    }

    fn check_borders(&mut self) {
        let [width, height] = [WIDTH as f32, HEIGHT as f32];
        let [x, y] = self.position.to_array();
        if x > width / 2.0 {
            self.position.x = -width / 2.0;
        } else if x < -width / 2.0 {
            self.position.x = width / 2.0;
        }
        if y > height / 2.0 {
            self.position.y = -height / 2.0;
        } else if y < -height / 2.0 {
            self.position.y = height / 2.0;
        }
    }

    pub fn step(&mut self, kd_tree: &KdTree<Boid>) {
        self.flock(kd_tree);
        self.update();
        self.check_borders();
    }
}

impl KdPoint for Boid {
    type Scalar = f32;
    type Dim = typenum::U2;
    fn at(&self, k: usize) -> Self::Scalar {
        match k {
            0 => self.position.x,
            1 => self.position.y,
            _ => panic!("Invalid index"),
        }
    }
}

impl FromIterator<Boid> for [Boid; NUM_POINTS] {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Boid>,
    {
        iter.into_iter().collect()
    }
}
