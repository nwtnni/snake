extern crate termion;

use std::fmt;
use std::collections::HashMap;
use std::io::{Write, stdout, stdin};
use std::time::{Instant, Duration};
use std::thread;

use termion::{color, cursor, clear};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{AsyncReader, async_stdin};

#[derive(Copy, Clone)]
enum Dir {
    N,
    S,
    E,
    W,
}

type Pos = (i32, i32);

struct Segment {
    dir: Dir,
    pos: Pos,
}

struct Snake(Vec<Segment>);

#[derive(Copy, Clone)]
enum Fruit {
    Growth,
    Death,
    Speed,
    Slow,
}

enum GameError {
    OutOfBounds,
    SelfCollision,
    FruitDeath,
}

struct Game {
    /// Terminal size
    bounds: Pos,

    /// Snake location
    snake: Snake,

    /// Fruits on the board
    fruits: HashMap<Pos, Fruit>,

    /// Previous frame instant
    previous: Instant,

    /// Delay between frames
    delay: Duration,
}

impl fmt::Display for Fruit {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let (display, color) = match self {
        | Fruit::Growth => ('🍏', &color::Green    as &color::Color),
        | Fruit::Death  => ('🍉', &color::Red      as &color::Color),
        | Fruit::Speed  => ('🍒', &color::LightRed as &color::Color),
        | Fruit::Slow   => ('🍍', &color::Blue     as &color::Color),
        };

        write!(
            fmt,
            "{color}{display}{reset}",
            color = color::Fg(color),
            display = display,
            reset = color::Fg(color::Reset),
        )
    }
}

impl fmt::Display for Snake {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let Snake(segments) = self;
        let first = segments.first().unwrap();
        let (x, y) = first.pos;

        let display = match first.dir {
        | Dir::N | Dir::S => '│',
        | Dir::E | Dir::W => '─',
        };

        write!(
            fmt,
            "{goto}{color}{display}",
            goto = cursor::Goto(x as u16, y as u16),
            color = color::Fg(color::White),
            display = display,
        );

        for pair in segments.windows(2) {
            let (head, tail) = (&pair[0], &pair[1]);
            let (x, y) = tail.pos;

            let display = match (head.dir, tail.dir) {
            | (Dir::N, Dir::E)                    => '╯',
            | (Dir::N, Dir::W)                    => '╰',
            | (Dir::S, Dir::E)                    => '╮',
            | (Dir::S, Dir::W)                    => '╭',
            | (Dir::W, Dir::W) | (Dir::E, Dir::E) => '─',
            | (Dir::N, Dir::N) | (Dir::S, Dir::S) => '│',
            | _                                   => panic!("Illegal game state"),
            };

            write!(
                fmt,
                "{goto}{display}",
                goto = cursor::Goto(x as u16, y as u16),
                display = display,
            );
        }

        write!(fmt, "{}", color::Fg(color::Reset))
    }
}

impl Snake {
    fn step(&mut self, fruits: &mut HashMap<(i32, i32), Fruit>, (max_x, max_y): Pos, dir: Dir) -> Result<Option<Fruit>, GameError> {
        let (x, y) = self.0.first().unwrap().pos;
        let (x, y) = match dir {
        | Dir::N => (x    , y - 1),
        | Dir::S => (x    , y + 1),
        | Dir::E => (x + 1, y    ),
        | Dir::W => (x - 1, y    ),
        };

        // Bounds check
        if x < 0 || y < 0 || x > max_x || y > max_y {
            return Err(GameError::OutOfBounds)
        }

        let Snake(segments) = self;
        let fruit = fruits.get(&(x, y)).cloned();

        if let Some(Fruit::Growth) = fruit {} else { segments.pop(); }

        // Self collision check
        if segments.iter().any(|segment| segment.pos == (x, y)) {
            return Err(GameError::SelfCollision)
        }

        // Update body with new segment
        segments.insert(0, Segment { dir, pos: (x, y) });

        // Fruit check
        match fruit {
        | Some(Fruit::Death) => Err(GameError::FruitDeath),
        | None               => Ok(None),
        | _                  => Ok(fruits.remove(&(x, y)))
        }
    }
}

fn main() {

    let stdout = stdout();
    let mut stdin = async_stdin().keys();
    let mut stdout = stdout.lock()
        .into_raw_mode()
        .unwrap();

    let test = Duration::new(1, 0);

    loop {

        let mut event = stdin.next();

        while let Some(next) = stdin.next() {
            event = Some(next);
        }

        write!(stdout, "{:?}", event);


        if let Some(event) = event {
            match event.unwrap() {
            Key::Char('q') => break,
            _              => (),
            };
        };
        stdout.flush().unwrap();
        thread::sleep(test);
    }
}
