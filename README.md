# turtle-rs
Turtle Graphics for Rust. Generates Scalable Vector Graphics (SVG) out-of-the-box.

## Example

```rust
extern crate turtle;

use turtle::{Canvas, Turtle};

fn main() {
    let mut t = Canvas::new();
    // move the turtle 100.0 points upwards
    t.forward(100.0);
    // rotate the head of the turtle by 90 degree to the right
    t.right(90.0);
    // move 100.0 forward again (now to the right).
    t.forward(100.0);
    // ...

    // write the graphic (SVG) to stdout.
    t.save_svg(&mut std::io::stdout()).unwrap();
}
```

For more examples see my [Lindenmayer-system library][1].

[1]: https://github.com/mneumann/lindenmayer-system
