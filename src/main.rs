extern crate rand;
extern crate termion;

use std::fmt;
use std::collections::HashMap;
use std::io::{Write, stdout};
use std::time::Duration;
use std::thread;

use rand::Rng;
use termion::{color, cursor, clear};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::async_stdin;

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

#[derive(Debug)]
enum Ending {
    OutOfBounds,
    SelfCollision,
    FruitDeath,
    Victory,
    Quit,
}

struct Game {
    /// Terminal size
    bounds: Pos,

    /// Snake location
    snake: Snake,

    /// Current direction
    dir: Dir,

    /// Fruits on the board
    fruits: HashMap<Pos, Fruit>,

    /// Delay between frames
    delay: Duration,

    /// Points accrued
    points: i32,
}

impl fmt::Display for Fruit {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let (display, color) = match self {
        | Fruit::Growth => ('🍏', &color::Green    as &color::Color),
        | Fruit::Death  => ('💀', &color::Red      as &color::Color),
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
            | (Dir::N, Dir::E) | (Dir::W, Dir::S) => '╯',
            | (Dir::N, Dir::W) | (Dir::E, Dir::S) => '╰',
            | (Dir::S, Dir::E) | (Dir::W, Dir::N) => '╮',
            | (Dir::S, Dir::W) | (Dir::E, Dir::N) => '╭',
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

impl fmt::Display for Game {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {

        write!(fmt, "{}", clear::All);

        for ((x, y), fruit) in &self.fruits {
            write!(
                fmt,
                "{goto}{fruit}",
                goto = cursor::Goto(*x as u16, *y as u16),
                fruit = fruit,
            );
        }

        write!(fmt, "{}", self.snake)
    }
}

impl Snake {
    fn new(max_x: u16, max_y: u16) -> Self {
        Snake(vec![
            Segment {
                dir: Dir::N,
                pos: (max_x as i32/ 2, max_y as i32/ 2),
            }
        ])
    }

    fn step(&mut self, fruits: &mut HashMap<(i32, i32), Fruit>, (max_x, max_y): Pos, dir: Dir) -> Result<Option<Fruit>, Ending> {
        let (x, y) = self.0.first().unwrap().pos;
        let (x, y) = match dir {
        | Dir::N => (x    , y - 1),
        | Dir::S => (x    , y + 1),
        | Dir::E => (x + 1, y    ),
        | Dir::W => (x - 1, y    ),
        };

        // Bounds check
        if x < 0 || y < 0 || x > max_x || y > max_y {
            return Err(Ending::OutOfBounds)
        }

        let Snake(segments) = self;
        let fruit = fruits.get(&(x, y)).cloned();

        if let Some(Fruit::Growth) = fruit {} else { segments.pop(); }

        // Self collision check
        if self.contains((x, y)) {
            return Err(Ending::SelfCollision)
        }

        // Update body with new segment
        segments.insert(0, Segment { dir, pos: (x, y) });

        // Fruit check
        Ok(fruits.remove(&(x, y)))
    }

    fn contains(&self, (x, y): Pos) -> bool {
        let Snake(segments) = self;
        segments.iter().any(|segment| segment.pos == (x, y))
    }
}

fn main() {

    let mut rng = rand::thread_rng();
    let stdout = stdout();
    let mut stdin = async_stdin().keys();
    let mut stdout = stdout.lock()
        .into_raw_mode()
        .unwrap();

    let (x, y) = termion::terminal_size().unwrap();
    write!(stdout, "{}", cursor::Hide);

    let mut game = Game {
        bounds: (x as i32, y as i32),
        snake: Snake::new(x, y),
        dir: Dir::N,
        fruits: HashMap::default(),
        delay: Duration::from_millis(100),
        points: 0,
    };

    let ending = loop {

        thread::sleep(game.delay);
        let maybe = stdin.next();

        // Handle user input
        if maybe.is_some() {

            // Drain event queue completely if there's more than one event
            let mut event = maybe.unwrap();
            while let Some(next) = stdin.next() { event = next; }

            match event.unwrap() {
            | Key::Char('w') | Key::Up    => game.dir = Dir::N,
            | Key::Char('a') | Key::Left  => game.dir = Dir::W,
            | Key::Char('s') | Key::Down  => game.dir = Dir::S,
            | Key::Char('d') | Key::Right => game.dir = Dir::E,
            | Key::Char('q') | Key::Esc   => break Ending::Quit,
            | _                           => (),
            };
        }

        // Randomly spawn fruits
        if rng.gen_range(0, 10 * (game.fruits.len() + 1)) == 0 {
            let fruit = match rng.gen_range(0, 10) {
            | 0 => Fruit::Death,
            | 2 => Fruit::Speed,
            | 3 => Fruit::Slow,
            | _ => Fruit::Growth,
            };

            let (max_x, max_y) = game.bounds;
            let (mut x, mut y) = (rng.gen_range(0, max_x), rng.gen_range(0, max_y));
            while game.snake.contains((x, y)) {
                x = rng.gen_range(0, max_x);
                y = rng.gen_range(0, max_y);
            }

            game.fruits.insert((x, y), fruit);
        }

        // Move the snake!
        match game.snake.step(&mut game.fruits, game.bounds, game.dir) {
        | Err(err)                => break err,
        | Ok(Some(Fruit::Death))  => break Ending::FruitDeath,
        | Ok(Some(Fruit::Growth)) => game.points += 10,
        | Ok(Some(Fruit::Speed))  => game.delay  /= 2,
        | Ok(Some(Fruit::Slow))   => game.delay  *= 2,
        | Ok(None)                => (),
        }

        write!(stdout, "{}", game);
        stdout.flush().unwrap();
    };

    write!(stdout, "{}{}{}", cursor::Goto(1, 1), clear::All, cursor::Show);
    println!("{:?}\r", ending);
}
