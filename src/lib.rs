use std::io::{self, Write};
use std::f32::consts::PI;
use std::f32::INFINITY;
use std::ops::Neg;

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

impl Into<Degree> for f32 {
    fn into(self) -> Degree {
        Degree(self)
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

impl Neg for Distance {
    type Output = Distance;
    fn neg(self) -> Self::Output {
        Distance(-self.0)
    }
}

impl Neg for Degree {
    type Output = Degree;
    fn neg(self) -> Self::Output {
        Degree(-self.0)
    }
}

pub trait Turtle {
    /// Move turtle forward by specified `distance`.
    fn forward<T: Into<Distance>>(&mut self, distance: T);

    /// Move turtle backward by specified `distance`.
    fn backward<T: Into<Distance>>(&mut self, distance: T) {
        self.forward(-distance.into())
    }

    /// Move turtle forward by specified `distance` *without* drawing.
    fn move_forward<T: Into<Distance>>(&mut self, distance: T);

    /// Rotate around `angle`. If `angle` is positive,
    /// the turtle is turned to the left, if negative,
    /// to the right.
    fn rotate<T: Into<Degree>>(&mut self, angle: T);

    /// Turn turtle right by `angle` degree.
    fn right<T: Into<Degree>>(&mut self, angle: T) {
        self.rotate(-angle.into());
    }

    /// Turn turtle left by `angle` degree.
    fn left<T: Into<Degree>>(&mut self, angle: T) {
        self.rotate(angle.into());
    }

    /// Returns `true` if pen is down.
    fn is_pen_down(&self) -> bool;

    /// Returns `true` if pen is up.
    fn is_pen_up(&self) -> bool {
        !self.is_pen_down()
    }

    /// Put the pen down.
    fn pen_down(&mut self);

    /// Put the pen up.
    fn pen_up(&mut self);

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
struct TurtleState {
    pos: Position,
    angle: Degree,
    pendown: bool,
}

pub struct Canvas {
    states: Vec<TurtleState>,
    paths: Vec<Vec<Position>>,
}

impl Canvas {
    pub fn new() -> Canvas {
        let init_pos = Position::origin();
        let init_state = TurtleState {
            pos: init_pos,
            // The SVG coordinates are from top to bottom, while turtle coordinates are bottom to
            // top.
            angle: Degree(180.0), // points upwards
            pendown: true, /* start with pen down */
        };
        Canvas {
            states: vec![init_state],
            paths: vec![vec![init_pos]],
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
        let (sin, cos) = rad.0.sin_cos();
        let dx = sin * distance.0;
        let dy = cos * distance.0;
        (dx, dy)
    }

    fn line_to(&mut self, dst: Position) {
        self.paths.last_mut().unwrap().push(dst);
    }

    fn move_to(&mut self, dst: Position) {
        if self.paths.is_empty() {
            self.paths.push(vec![dst]);
        } else {
            let begin_new_path = self.paths.last().unwrap().len() > 1;
            if begin_new_path {
                self.paths.push(vec![dst]);
            } else {
                // Replace first path element with current position
                self.paths.last_mut().unwrap()[0] = dst;
            }
        }
    }

    /// Saves the turtle graphic as Scalable Vector Graphic (SVG).
    pub fn save_svg<W: Write>(&self, wr: &mut W) -> io::Result<()> {
        // Determine extend of canvas
        let mut min = Position(INFINITY, INFINITY);
        let mut max = Position(-INFINITY, -INFINITY);
        for path in self.paths.iter() {
            for pt in path.iter() {
                min.0 = min.0.min(pt.0).min(pt.0);
                max.0 = max.0.max(pt.0).max(pt.0);

                min.1 = min.1.min(pt.1).min(pt.1);
                max.1 = max.1.max(pt.1).max(pt.1);
            }
        }
        let (min_width, min_height) = (100.0, 100.0);
        let width = (max.0 - min.0).abs().max(min_width);
        let height = (max.1 - min.1).abs().max(min_height);
        let border_percent = 0.1;

        let top_left = Position(min.0 - border_percent * width,
                                min.1 - border_percent * height);

        let scale = 1.0 + 2.0 * border_percent;

        try!(writeln!(wr,
                      r#"<?xml version="1.0" encoding="UTF-8"?>
                <svg xmlns="http://www.w3.org/2000/svg"
                version="1.1" baseProfile="full"
                viewBox="{} {} {} {}">"#,
                      top_left.0,
                      top_left.1,
                      scale * width,
                      scale * height));

        // use a stroke width of 0.1% of the width or height of the canvas
        let stroke_width = scale * width.max(height) / 1000.0;
        try!(writeln!(wr,
                      r#"<g stroke="black" stroke-width="{}" fill="none">"#,
                      stroke_width));

        for path in self.paths.iter() {
            if let Some((head, tail)) = path.split_first() {
                try!(write!(wr, r#"<path d="M{} {}"#, head.0, head.1));
                for pos in tail {
                    try!(write!(wr, r#" L{} {}"#, pos.0, pos.1));
                }
                try!(writeln!(wr, r#"" />"#));
            }
        }
        try!(writeln!(wr, r#"</g>"#));

        writeln!(wr, "</svg>")
    }
}

impl Turtle for Canvas {
    /// Move turtle forward by specified `distance`.
    fn forward<T: Into<Distance>>(&mut self, distance: T) {
        let (dx, dy) = self.direction(distance.into());
        let src: Position = self.current_state().pos;
        let dst = Position(src.0 + dx, src.1 + dy);
        if self.is_pen_down() {
            self.line_to(dst);
        }
        self.current_state_mut().pos = dst;
    }

    fn rotate<T: Into<Degree>>(&mut self, angle: T) {
        let angle: Degree = angle.into();
        self.current_state_mut().angle.0 += angle.0;
    }

    fn move_forward<T: Into<Distance>>(&mut self, distance: T) {
        let (dx, dy) = self.direction(distance.into());
        let src: Position = self.current_state().pos;
        let dst = Position(src.0 + dx, src.1 + dy);
        self.move_to(dst);
    }

    fn is_pen_down(&self) -> bool {
        self.current_state().pendown
    }

    /// Put the pen down.
    fn pen_down(&mut self) {
        let pos = self.current_state().pos;
        self.move_to(pos);
        self.current_state_mut().pendown = true;
    }

    /// Put the pen up.
    fn pen_up(&mut self) {
        self.current_state_mut().pendown = false;
    }

    /// Positions the turtle exactly at `position`.
    fn goto(&mut self, position: Position) {
        self.current_state_mut().pos = position;
        self.move_to(position);
    }

    /// Push current turtle state on stack.
    fn push(&mut self) {
        let state = self.current_state_mut().clone();
        self.states.push(state);
    }

    /// Restore previously saved turtle state.
    fn pop(&mut self) {
        self.states.pop();
        let pos = self.current_state().pos;
        self.move_to(pos);
    }
}
