use crossterm::terminal;
use crossterm::{
    cursor::{position, MoveTo},
    event::{poll, read, Event, KeyCode},
    execute,
};
use rand::{rngs::ThreadRng, Rng};
use std::{
    fmt,
    io::{self, stdout, Write},
    time::Duration,
};

#[derive(PartialEq, Clone, Copy)]
enum DropSize {
    SMALL,
    MEDIUM,
    LARGE,
}

impl DropSize {
    fn from_int(i: u16) -> DropSize {
        match i {
            0 => DropSize::SMALL,
            1 => DropSize::MEDIUM,
            2 => DropSize::LARGE,
            _ => panic!("Unknown drop size"),
        }
    }
}

impl fmt::Display for DropSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DropSize::SMALL => write!(f, "."),
            DropSize::MEDIUM => write!(f, ":"),
            DropSize::LARGE => write!(f, "|"),
        }
    }
}

//#[derive(Copy, Clone)]
struct Drop {
    drop_size: DropSize,
    speed: u16,
    x: u16,
    y: u16,
}

impl Drop {
    fn new(cols: u16, rng: &mut ThreadRng) -> Self {
        let size = generate_drop_size();
        Drop {
            drop_size: size.clone(),
            speed: generate_speed(size),
            x: generate_random_number(0 as u16, cols, rng),
            y: 0,
        }
    }

    fn tick(&mut self) {
        self.y += self.speed;
    }

    fn render(&self) {
        let _ = execute!(io::stdout(), MoveTo(self.x, self.y));
        if self.drop_size == DropSize::SMALL {
            let _ = execute!(
                io::stdout(),
                crossterm::style::SetForegroundColor(crossterm::style::Color::DarkGrey)
            );
        } else {
            let _ = execute!(
                io::stdout(),
                crossterm::style::SetForegroundColor(crossterm::style::Color::White)
            );
        }
        print!("{}", self.drop_size);
    }
}

fn generate_drop_size() -> DropSize {
    let mut rng = rand::thread_rng();
    DropSize::from_int(generate_random_number(0 as u16, 3 as u16, &mut rng))
}

fn generate_speed(size: DropSize) -> u16 {
    if size == DropSize::LARGE {
        return 1;
    } else if size == DropSize::MEDIUM {
        return 2;
    } else {
        return 3;
    }
}

fn generate_random_number(min: u16, max: u16, rng: &mut ThreadRng) -> u16 {
    rng.gen_range(min.into()..max.into())
}

fn flush_resize_events(first_resize: (u16, u16)) -> (u16, u16) {
    let mut last_resize = first_resize;
    while let Ok(true) = poll(Duration::from_millis(50)) {
        if let Ok(Event::Resize(x, y)) = read() {
            last_resize = (x, y);
        }
    }
    last_resize
}

fn add_new_drops(drops: &mut Vec<Drop>, cols: u16) {
    let max_num_drops = 30;
    if drops.len() < max_num_drops {
        for _ in 0..(max_num_drops - drops.len()) {
            let mut rng = rand::thread_rng();
            let should_create = generate_random_number(1, 11, &mut rng);
            if should_create > 5 {
                drops.push(Drop::new(cols, &mut rng));
            }
        }
    }
}

fn main() {
    let window_size = crossterm::terminal::size().unwrap();
    let mut cols = window_size.0;
    let mut rows = window_size.1;
    let mut loop_time = Duration::from_millis(41);
    let loop_time_step = Duration::from_millis(10);
    let mut drops: Vec<Drop> = Vec::new();
    let _ = execute!(io::stdout(), terminal::EnterAlternateScreen);
    let _ = execute!(io::stdout(), crossterm::cursor::Hide);
    loop {
        if poll(loop_time).expect("Failed to poll") {
            // It's guaranteed that read() wont block if `poll` returns `Ok(true)`
            let event = read().expect("Could not read");

            if event == Event::Key(KeyCode::Char('c').into()) {
                println!("Cursor position: {:?}\r", position());
            }
            if event == Event::Key(KeyCode::Char('+').into()) && loop_time > loop_time_step {
                loop_time -= loop_time_step;
            }
            if event == Event::Key(KeyCode::Char('-').into()) {
                loop_time += loop_time_step;
            }

            if let Event::Resize(x, y) = event {
                let new_size = flush_resize_events((x, y));
                cols = new_size.0;
                rows = new_size.1;
            }

            if event == Event::Key(KeyCode::Char('q').into()) {
                let _ = execute!(io::stdout(), terminal::LeaveAlternateScreen);
                break;
            }
        } else {
            drops = drops
                .into_iter()
                .map(|mut drop| {
                    drop.tick();
                    drop
                })
                .filter(|drop| drop.y < rows.into())
                .collect();

            let _ = execute!(
                io::stdout(),
                crossterm::terminal::Clear(terminal::ClearType::All)
            );
            drops.iter().for_each(|drop| drop.render());

            add_new_drops(&mut drops, cols);

            let _ = stdout().flush();
        }
    }
    let _ = execute!(io::stdout(), crossterm::cursor::Show);
}
