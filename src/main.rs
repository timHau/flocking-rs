extern crate kd_tree;
extern crate nannou;

mod boid;
mod system;

use self::system::FlockingSystem;
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
    system: FlockingSystem,
}

fn model(app: &App) -> Model {
    Model {
        system: FlockingSystem::new(app),
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.system.update();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    model.system.show(&draw);
    draw.to_frame(app, &frame).unwrap();
}
