extern crate nannou;

mod point2d;

use self::point2d::{Boid, NUM_POINTS};
use nannou::prelude::*;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

fn main() {
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .size(WIDTH, HEIGHT)
        .run();
}

struct Model {
    boids: [Boid; NUM_POINTS],
    texture: wgpu::Texture,
}

fn model(app: &App) -> Model {
    let points = (0..NUM_POINTS)
        .map(|_| Boid::rand_new(app))
        .collect::<Vec<_>>();

    let assets = app.assets_path().unwrap();
    let img_path = assets.join("cursor.png");
    let texture = wgpu::Texture::from_path(app, img_path).unwrap();

    Model {
        boids: points.try_into().unwrap(),
        texture,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let all_boids = model.boids.clone();
    for i in 0..NUM_POINTS {
        let boid = &mut model.boids[i];
        boid.step(&all_boids, app);
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    for point in &model.boids {
        point.show(&draw, &model.texture);
    }

    draw.to_frame(app, &frame).unwrap();
}
