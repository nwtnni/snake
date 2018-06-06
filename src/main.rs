extern crate termion;

use std::fmt;
use std::collections::HashMap;

use termion::{color, cursor, clear};

#[derive(Copy, Clone)]
enum Dir {
    N,
    S,
    E,
    W,
}

struct Segment {
    dir: Dir,
    pos: (i32, i32),
}

struct Snake {
    bounds: (i32, i32),
    body: Vec<Segment>,
}

enum GameError {
    OutOfBounds,
    SelfCollision,
    FruitDeath,
}

#[derive(Copy, Clone)]
enum Fruit {
    Growth,
    Death,
    Speed,
    Slow,
}

impl fmt::Display for Snake {

    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {

        let segments = &self.body;
        let first = segments.first().unwrap();
        let (x, y) = first.pos;

        let display = match first.dir {
        | Dir::N | Dir::S => '│',
        | Dir::E | Dir::W => '─',
        };

        write!(
            fmt,
            "{goto}{display}",
            goto = cursor::Goto(x as u16, y as u16),
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

        Ok(())
    }
}

impl Snake {
    fn step(&mut self, fruits: &mut HashMap<(i32, i32), Fruit>, dir: Dir) -> Result<Option<Fruit>, GameError> {
        let (max_x, max_y) = self.bounds;
        let (x, y) = self.body.first().unwrap().pos;
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

        let segments = &mut self.body;
        let fruit = fruits.get(&(x, y)).cloned();

        if let Some(Fruit::Growth) = fruit {} else { segments.pop(); }

        // Self collision check
        if segments.iter().any(|segment| segment.pos == (x, y)) {
            return Err(GameError::SelfCollision)
        }

        // Fruit check
        match fruit {
        | Some(Fruit::Death) => Err(GameError::FruitDeath),
        | None               => Ok(None),
        | _                  => Ok(fruits.remove(&(x, y)))
        }
    }
}

fn main() {
    println!("Hello, world!");
}
