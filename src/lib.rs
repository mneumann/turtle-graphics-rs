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
struct TurtleState {
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

pub struct Canvas {
    states: Vec<TurtleState>,
    paths: Vec<Vec<Position>>,
}

impl Canvas {
    pub fn new() -> Canvas {
        Canvas {
            states: vec![TurtleState::new()],
            paths: vec![],
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
        let dx = cos * distance.0;
        let dy = sin * distance.0;
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
        let width = (max.0 - min.0).abs().max(10.0);
        let height = (max.1 - min.1).abs().max(10.0);

        println!("width: {}", width);
        println!("height: {}", height);

        let top_left = Position(min.0 - width / 10.0, min.1 - height / 10.0);
        let bottom_right = Position(max.0 + width / 10.0, max.1 + height / 10.0);

        try!(writeln!(wr,
                      r#"<?xml version="1.0" encoding="UTF-8"?>
                <svg xmlns="http://www.w3.org/2000/svg"
                version="1.1" baseProfile="full"
                viewBox="{} {} {} {}">"#,
                      top_left.0.ceil(),
                      top_left.1.ceil(),
                      bottom_right.0.ceil(),
                      bottom_right.1.ceil()));

         try!(writeln!(wr, r#"<g stroke="black" stroke-width="1" fill="none">"#));
        
        for path in self.paths.iter() {
            if let Some((head, tail)) = path.split_first() {
                try!(write!(wr, r#"<path d="M{} {}"#, head.0.round(), head.1.round()));
                for pos in tail {
                    try!(write!(wr, r#" L{} {}"#, pos.0.round(), pos.1.round()));
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
        let pendown = self.current_state().pendown;
        let src: Position = self.current_state().pos;
        let dst = Position(src.0 + dx, src.1 + dy);
        if pendown {
            self.line_to(dst);
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
        let pos = self.current_state().pos;
        self.move_to(pos);
        self.current_state_mut().pendown = true;
    }

    /// Put the pen up.
    fn penup(&mut self) {
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
