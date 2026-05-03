use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use colored::Colorize;

const COLORS: [(u8, u8, u8); 9] = [
    (100, 220, 80),  // green
    (50, 200, 200),  // cyan
    (255, 220, 50),  // yellow
    (60, 130, 255),  // blue
    (255, 150, 40),  // orange
    (255, 80, 80),   // red
    (140, 80, 255),  // purple
    (220, 80, 255),  // magenta
    (255, 100, 180), // pink
];
static ID_FOR_COLOR: LazyLock<Mutex<HashMap<i32, (u8, u8, u8)>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub fn log(source_id: i32, severity: Severity, contents: &str) {
    let mut map = ID_FOR_COLOR.lock().unwrap();
    let len = map.len();
    if !map.contains_key(&source_id) {
        if source_id == 0 {
            map.insert(source_id, (255, 255, 255));
        } else {
            if len <= COLORS.len() {
                map.insert(source_id, COLORS[len - 1]);
            } else {
                map.insert(source_id, (255, 255, 255));
            }
        }
    }

    if severity < LOG_LEVEL {
        return;
    }

    let Some(col) = map.get(&source_id) else {
        panic!("");
    };

    match severity {
        Severity::Info => println!("{}:{}", "[I]".white().underline(), contents.truecolor(col.0, col.1, col.2)),
        Severity::Warning => println!("{}:{}", "[W]".yellow().underline(), contents.truecolor(col.0, col.1, col.2)),
        Severity::Error => eprintln!("{}:{}", "[E]".red().underline(), contents.truecolor(col.0, col.1, col.2)),
    }
}
pub fn log_unformated(contents: &str) {
    println!("{}", contents.white());
}

static LOG_LEVEL: Severity = Severity::Info;

#[derive(PartialEq, PartialOrd, Eq)]
pub enum Severity {
    Info = 0,
    Warning = 1,
    Error = 2,
}
