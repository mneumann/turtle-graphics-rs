use std::io::{self, Write};
use std::f32::consts::PI;
use std::f32::INFINITY;


#[derive(Copy, Clone, Debug)]
pub struct Position(f32, f32);

impl Position {
    pub fn origin() -> Position {
        Position(0.0, 0.0)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Degree(pub f32);

#[derive(Copy, Clone, Debug)]
pub struct Radiant(pub f32);

impl Into<Degree> for Radiant {
    fn into(self) -> Degree {
        Degree(self.0 * 180.0 / PI)
    }
}

impl Into<Radiant> for Degree {
    fn into(self) -> Radiant {
        Radiant(self.0 * PI / 180.0)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Distance(f32);

impl Into<Distance> for f32 {
    fn into(self) -> Distance {
        Distance(self)
    }
}

pub trait Turtle {
    /// Move turtle forward by specified `distance`.
    fn forward<T: Into<Distance>>(&mut self, distance: T);

    /// Move turtle backward by specified `distance`.
    fn backward<T: Into<Distance>>(&mut self, distance: T);

    /// Turn turtle right by `angle` degree.
    fn right<T: Into<Degree>>(&mut self, angle: T);

    /// Turn turtle left by `angle` degree.
    fn left<T: Into<Degree>>(&mut self, angle: T);

    /// Put the pen down.
    fn pendown(&mut self);

    /// Put the pen up.
    fn penup(&mut self);

    fn goto(&mut self, pos: Position);

    fn home(&mut self) {
        self.goto(Position::origin());
    }

    /// Push current turtle state on stack.
    fn push(&mut self);

    /// Restore previously saved turtle state.
    fn pop(&mut self);
}

#[derive(Clone)]
pub struct TurtleState {
    pos: Position,
    angle: Degree,
    pendown: bool,
}

impl TurtleState {
    fn new() -> TurtleState {
        TurtleState {
            pos: Position::origin(),
            angle: Degree(0.0),
            pendown: false,
        }
    }
}

pub struct TurtleRecorder {
    states: Vec<TurtleState>,
    lines: Vec<(Position, Position)>,
}

impl TurtleRecorder {
    pub fn new() -> TurtleRecorder {
        TurtleRecorder {
            states: vec![TurtleState::new()],
            lines: vec![],
        }
    }

    #[inline]
    fn current_state_mut(&mut self) -> &mut TurtleState {
        self.states.last_mut().unwrap()
    }

    #[inline]
    fn current_state(&self) -> &TurtleState {
        self.states.last().unwrap()
    }

    #[inline]
    fn direction(&self, distance: Distance) -> (f32, f32) {
        let state = self.current_state();
        let rad: Radiant = state.angle.into();
        let dx = rad.0.cos() * distance.0;
        let dy = rad.0.sin() * distance.0;
        (dx, dy)
    }

    fn draw_line(&mut self, src: Position, dst: Position) {
        self.lines.push((src, dst));
    }

    /// Saves the turtle graphic as Scalable Vector Graphic (SVG).
    pub fn save_as_svg<W: Write>(&self, wr: &mut W) -> io::Result<()> {
        // Determine extend of canvas
        let mut min = Position(INFINITY, INFINITY);
        let mut max = Position(-INFINITY, -INFINITY);
        for &(ref src, ref dst) in self.lines.iter() {
            min.0 = min.0.min(src.0).min(dst.0);
            max.0 = max.0.max(src.0).max(dst.0);

            min.1 = min.1.min(src.1).min(dst.1);
            max.1 = max.1.max(src.1).max(dst.1);

        }
        let width = (max.0 - min.0).abs();
        let height = (max.1 - min.1).abs();

        let top_left = Position(min.0 - width / 10.0, min.1 - height / 10.0);
        let bottom_right = Position(max.0 + width / 10.0, max.1 + height / 10.0);

        try!(writeln!(wr,
                      r#"<?xml version="1.0" encoding="UTF-8"?>
                <svg xmlns="http://www.w3.org/2000/svg"
                version="1.1" baseProfile="full"
                width="100%" height="100%"
                viewBox="{} {} {} {}">"#,
                      top_left.0,
                      top_left.1,
                      bottom_right.0,
                      bottom_right.1));

        for &(ref src, ref dst) in self.lines.iter() {
            try!(writeln!(wr,
                          r#"<path d="M{} {} L{} {}" stroke="black" stroke-width="1"/>"#,
                          src.0,
                          src.1,
                          dst.0,
                          dst.1));
        }

        writeln!(wr, "</svg>")
    }
}

impl Turtle for TurtleRecorder {
    /// Move turtle forward by specified `distance`.
    fn forward<T: Into<Distance>>(&mut self, distance: T) {
        let (dx, dy) = self.direction(distance.into());
        let pendown = self.current_state().pendown;
        let src: Position = self.current_state().pos;
        let dst = Position(src.0 + dx, src.1 + dy);
        if pendown {
            self.draw_line(src, dst);
        }
        self.current_state_mut().pos = dst;
    }

    /// Move turtle backward by specified `distance`.
    fn backward<T: Into<Distance>>(&mut self, distance: T) {
        let distance: Distance = distance.into();
        self.forward(Distance(-distance.0))
    }

    /// Turn turtle right by `angle` degrees.
    fn right<T: Into<Degree>>(&mut self, angle: T) {
        let angle: Degree = angle.into();
        self.current_state_mut().angle.0 += angle.0;
    }

    /// Turn turtle left by `angle` degrees.
    fn left<T: Into<Degree>>(&mut self, angle: T) {
        let angle: Degree = angle.into();
        self.current_state_mut().angle.0 -= angle.0;
    }

    /// Put the pen down.
    fn pendown(&mut self) {
        self.current_state_mut().pendown = true;
    }

    /// Put the pen up.
    fn penup(&mut self) {
        self.current_state_mut().pendown = false;
    }

    fn goto(&mut self, pos: Position) {
        self.current_state_mut().pos = pos;
    }

    /// Push current turtle state on stack.
    fn push(&mut self) {
        let state = self.current_state_mut().clone();
        self.states.push(state);
    }

    /// Restore previously saved turtle state.
    fn pop(&mut self) {
        self.states.pop();
    }
}


fn main() {
    use std::fs::File;
    let mut t = TurtleRecorder::new();
    t.pendown();
    t.forward(10.0);
    t.right(Degree(90.0));
    t.forward(10.0);
    t.right(Degree(90.0));
    t.forward(10.0);
    t.right(Degree(90.0));
    t.forward(10.0);
    t.save_as_svg(&mut File::create("test.svg").unwrap());
}
