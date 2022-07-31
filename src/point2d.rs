use nannou::prelude::*;
use rand::Rng;
use wgpu::Texture;

pub const NUM_POINTS: usize = 500;
const MAX_FORCE: f32 = 0.03;
const MAX_SPEED: f32 = 2.8;
const DESIRED_SEPARATION: f32 = 30.0;
const ALIGN_NEIGHBOR_DISTANCE: f32 = 50.0;
const COHESION_NEIGHBOR_DISTANCE: f32 = 50.0;
const BOID_RADIUS: f32 = 20.0;

#[derive(Clone, Debug)]
pub struct Boid {
    position: Point2,
    velocity: Point2,
    acceleration: Point2,
    radius: f32,
}

impl Boid {
    pub fn rand_new(app: &App) -> Self {
        let [width, height] = app.window_rect().wh().to_array();
        let mut rng = rand::thread_rng();
        let rand_range_width = (-width / 2.0)..=(width / 2.0);
        let rand_range_height = (-height / 2.0)..=(height / 2.0);
        let position = Point2::new(
            rng.gen_range(rand_range_width),
            rng.gen_range(rand_range_height),
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

    fn separate(&self, boids: &[Boid]) -> Point2 {
        let mut steer = Point2::new(0.0, 0.0);
        let mut count = 0;
        for other in boids {
            let d = self.position - other.position;
            let distance = d.length();
            if distance > 0.0 && distance < DESIRED_SEPARATION {
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

    fn align(&self, boids: &[Boid]) -> Point2 {
        let mut sum = Point2::new(0.0, 0.0);
        let mut count = 0;
        for other in boids {
            let d = self.position - other.position;
            let distance = d.length();
            if distance > 0.0 && distance < ALIGN_NEIGHBOR_DISTANCE {
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

    fn cohesion(&self, boids: &[Boid]) -> Point2 {
        let mut sum = Point2::new(0.0, 0.0);
        let mut count = 0;
        for other in boids {
            let d = self.position - other.position;
            let distance = d.length();
            if distance > 0.0 && distance < COHESION_NEIGHBOR_DISTANCE {
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

    fn flock(&mut self, boids: &[Boid]) {
        let mut separation = self.separate(boids);
        let mut alignment = self.align(boids);
        let mut cohesion = self.cohesion(boids);

        // weight forces
        separation = separation * 1.9;
        alignment = alignment * 1.0;
        cohesion = cohesion * 1.0;

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

    fn check_borders(&mut self, app: &App) {
        let [width, height] = app.window_rect().wh().to_array();
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

    pub fn step(&mut self, boids: &[Boid], app: &App) {
        self.flock(boids);
        self.update();
        self.check_borders(app);
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
