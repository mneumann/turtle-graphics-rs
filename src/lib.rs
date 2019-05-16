use std::io::{self, Write};
use std::f32::consts::PI;
use std::ops::{Add, Neg};

#[derive(Copy, Clone, Debug)]
pub struct Position(f32, f32);

impl Position {
    pub fn origin() -> Position {
        Position(0.0, 0.0)
    }

    pub fn min(&self, other: &Position) -> Position {
        Position(self.0.min(other.0), self.1.min(other.1))
    }

    pub fn max(&self, other: &Position) -> Position {
        Position(self.0.max(other.0), self.1.max(other.1))
    }

    pub fn min_max(&self, min_max: &(Position, Position)) -> (Position, Position) {
        (self.min(&min_max.0), self.max(&min_max.1))
    }
}

struct Bounds {
    min_max: Option<(Position, Position)>,
}

impl Bounds {
    fn new() -> Bounds {
        Bounds { min_max: None }
    }

    fn add_position(&mut self, pos: Position) {
        let mm = match self.min_max {
            None => (pos, pos),
            Some(ref a) => pos.min_max(a),
        };

        self.min_max = Some(mm);
    }

    #[allow(dead_code)]
    fn is_bounded(&self) -> bool {
        self.min_max.is_some()
    }

    fn width(&self) -> f32 {
        let (min, max) = self.min_max.unwrap();
        (max.0 - min.0).abs()
    }

    fn height(&self) -> f32 {
        let (min, max) = self.min_max.unwrap();
        (max.1 - min.1).abs()
    }

    fn min_x(&self) -> f32 {
        let (min, _) = self.min_max.unwrap();
        min.0
    }
    fn min_y(&self) -> f32 {
        let (min, _) = self.min_max.unwrap();
        min.1
    }

    fn max_x(&self) -> f32 {
        let (_, max) = self.min_max.unwrap();
        max.0
    }
    fn max_y(&self) -> f32 {
        let (_, max) = self.min_max.unwrap();
        max.1
    }
}

impl Add<Position> for Position {
    type Output = Position;
    fn add(self, other: Position) -> Self::Output {
        Position(self.0 + other.0, self.1 + other.1)
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
            // The coordinate system we use: x from left to right. y from bottom to top.
            angle: Degree(0.0), // points upwards
            pendown: true, // start with pen down
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
        let dx = -sin * distance.0;
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

    fn foreach_position<F: FnMut(Position)>(&self, mut f: F, scale_x: f32, scale_y: f32) {
        for path in self.paths.iter() {
            for pos in path.iter() {
                f(Position(pos.0 * scale_x, pos.1 * scale_y));
            }
        }
    }

    /// Saves the turtle graphic as Embedded Postscript (EPS)
    pub fn save_eps<W: Write>(&self, wr: &mut W) -> io::Result<()> {
        // Determine extend of canvas
        let mut bounds = Bounds::new();

        // The EPS coordinates are from bottom to top, like turtle coordinates.
        self.foreach_position(|pos| bounds.add_position(pos), 1.0, 1.0);

        let (min_width, min_height) = (100.0, 100.0);
        let width = bounds.width().max(min_width);
        let height = bounds.height().max(min_height);
        let border_percent = 0.1;

        let scale = 1.0 + 2.0 * border_percent;

        writeln!(wr,
                      r#"%%!PS-Adobe-3.0 EPSF-3.0
%%Creator: https://github.com/mneumann/turtle-graphics-rs
%%DocumentData: Clean7Bit
%%Origin: 0 0
%%BoundingBox: {} {} {} {}
%%LanguageLevel: 2
%%Pages: 1
%%Page: 1 1
"#,
                      bounds.min_x() - border_percent * width,
                      bounds.min_y() - border_percent * height,
                      bounds.max_x() + border_percent * width,
                      bounds.max_y() + border_percent * height)?;

        // use a stroke width of 0.1% of the width or height of the canvas
        let stroke_width = scale * width.max(height) / 1000.0;
        writeln!(wr, r#"{} setlinewidth"#, stroke_width)?;

        for path in self.paths.iter() {
            if let Some((head, tail)) = path.split_first() {
                writeln!(wr, "newpath")?;
                writeln!(wr, "  {} {} moveto", head.0, head.1)?;
                for pos in tail {
                    writeln!(wr, r#"  {} {} lineto"#, pos.0, pos.1)?;
                }
                writeln!(wr, r#"stroke"#)?;
            }
        }
        writeln!(wr, "%%EOF")
    }


    /// Saves the turtle graphic as Scalable Vector Graphic (SVG).
    pub fn save_svg<W: Write>(&self, wr: &mut W) -> io::Result<()> {
        // Determine extend of canvas
        let mut bounds = Bounds::new();

        // The SVG coordinates are from top to bottom, while turtle coordinates are
        // bottom to
        // top. We have to convert between the two. (multiply `y` by -1.0)
        self.foreach_position(|pos| bounds.add_position(pos), 1.0, -1.0);

        let (min_width, min_height) = (100.0, 100.0);
        let width = bounds.width().max(min_width);
        let height = bounds.height().max(min_height);
        let border_percent = 0.1;

        let top_left = Position(bounds.min_x() - border_percent * width,
                                bounds.min_y() - border_percent * height);

        let scale = 1.0 + 2.0 * border_percent;

        writeln!(wr,
                      r#"<?xml version="1.0" encoding="UTF-8"?>
                <svg xmlns="http://www.w3.org/2000/svg"
                version="1.1" baseProfile="full"
                viewBox="{} {} {} {}">"#,
                      top_left.0,
                      top_left.1,
                      scale * width,
                      scale * height)?;

        // use a stroke width of 0.1% of the width or height of the canvas
        let stroke_width = scale * width.max(height) / 1000.0;
        writeln!(wr,
                      r#"<g stroke="black" stroke-width="{}" fill="none">"#,
                      stroke_width)?;

        for path in self.paths.iter() {
            if let Some((head, tail)) = path.split_first() {
                // XXX
                let head = Position(head.0, -1.0 * head.1);

                write!(wr, r#"<path d="M{} {}"#, head.0, head.1)?;
                for pos in tail {
                    let pos = Position(pos.0, -1.0 * pos.1);
                    write!(wr, r#" L{} {}"#, pos.0, pos.1)?;
                }
                writeln!(wr, r#"" />"#)?;
            }
        }
        writeln!(wr, r#"</g>"#)?;

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
        self.current_state_mut().pos = dst;
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
