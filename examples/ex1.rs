extern crate turtle;

use std::fs::File;
use turtle::{Canvas, Turtle};

fn main() {
    let mut t = Canvas::new();
    t.forward(100.0);
    t.right(90.0);
    t.forward(100.0);
    t.pen_up();
    t.forward(10.0);
    t.pen_down();
    t.right(90.0);
    t.forward(100.0);
    t.right(90.0);
    t.forward(100.0);
    t.save_svg(&mut File::create("test.svg").unwrap()).unwrap();
}
