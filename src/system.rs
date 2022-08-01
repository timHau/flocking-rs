use super::boid::{Boid, NUM_POINTS};
use kd_tree::KdTree;
use nannou::{wgpu, App, Draw};

pub struct FlockingSystem {
    texture: wgpu::Texture,
    boids: [Boid; NUM_POINTS],
    kd_tree: KdTree<Boid>,
}

impl FlockingSystem {
    pub fn new(app: &App) -> Self {
        let boids = (0..NUM_POINTS)
            .map(|_| Boid::rand_new())
            .collect::<Vec<_>>();

        let assets = app.assets_path().unwrap();
        let img_path = assets.join("cursor.png");
        let texture = wgpu::Texture::from_path(app, img_path).unwrap();

        let kd_tree = KdTree::build_by_ordered_float(boids.clone());

        FlockingSystem {
            texture,
            boids: boids.try_into().unwrap(),
            kd_tree,
        }
    }

    pub fn update(&mut self) {
        for i in 0..NUM_POINTS {
            let boid = &mut self.boids[i];
            boid.step(&self.kd_tree);
        }
        // rebuild the kd tree after each step
        self.kd_tree = KdTree::build_by_ordered_float(self.boids.to_vec());
    }

    pub fn show(&self, draw: &Draw) {
        for boid in &self.boids {
            boid.show(draw, &self.texture);
        }
    }
}
