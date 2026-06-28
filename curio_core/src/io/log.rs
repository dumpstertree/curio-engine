use std::{
    collections::HashMap,
    default,
    sync::{LazyLock, Mutex},
};

static LOG_BUFFER: LazyLock<Mutex<Vec<(Severity, String)>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn get_and_clear_logs() -> Vec<(Severity, String)> {
    services().logger().get_and_clear_logs()

    // let mut buf = LOG_BUFFER.lock().unwrap();
    // std::mem::take(&mut *buf)
}
use colored::Colorize;

use crate::services;

// all available colors for source_ids
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

// level assigned for logging
static LOG_LEVEL: Severity = Severity::Info;

// all assigned ids for colors
static ID_FOR_COLOR: LazyLock<Mutex<HashMap<i32, (u8, u8, u8)>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// log a message. source_id corresponds to the owner and severity represents the log level
pub fn log(source_id: i32, severity: Severity, contents: &str) {
    services().logger().log(source_id, severity, contents);
    // // if severity is too low we ignore
    // if severity < LOG_LEVEL {
    //     return;
    // }

    // // get the map
    // let mut map = ID_FOR_COLOR.lock().unwrap();

    // // add a color for our new source_id
    // if !map.contains_key(&source_id) {
    //     // if source is 0 we use white
    //     if source_id == 0 {
    //         map.insert(source_id, (255, 255, 255));
    //     } else {
    //         // get a random color
    //         let len = map.len();
    //         if len <= COLORS.len() {
    //             map.insert(source_id, COLORS[len - 1]);
    //         } else {
    //             map.insert(source_id, (255, 255, 255));
    //         }
    //     }
    // }

    // // get the color
    // let Some(col) = map.get(&source_id) else {
    //     panic!("Failed to get Color for log");
    // };

    // // write line based on severity
    // match severity {
    //     Severity::Info => println!("{}:{}", "[I]".white().underline(), contents.truecolor(col.0, col.1, col.2)),
    //     Severity::Warning => println!("{}:{}", "[W]".yellow().underline(), contents.truecolor(col.0, col.1, col.2)),
    //     Severity::Error => eprintln!("{}:{}", "[E]".red().underline(), contents.truecolor(col.0, col.1, col.2)),
    // }
    // LOG_BUFFER
    //     .lock()
    //     .unwrap()
    //     .push((severity, contents.to_string()));
}

#[derive(PartialEq, PartialOrd, Eq, Clone)]
pub enum Severity {
    Info = 0,
    Warning = 1,
    Error = 2,
}

unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}

#[repr(C)]
pub struct Logger {
    log_buffer: Vec<(Severity, String)>,
}
impl Logger {
    pub fn new() -> Logger {
        Logger { log_buffer: Vec::new() }
    }
    pub fn get_and_clear_logs(&mut self) -> Vec<(Severity, String)> {
        let buf = self.log_buffer.to_vec();
        self.log_buffer.clear();
        buf
    }

    /// log a message. source_id corresponds to the owner and severity represents the log level
    pub fn log(&mut self, source_id: i32, severity: Severity, contents: &str) {
        // if severity is too low we ignore
        if severity < LOG_LEVEL {
            return;
        }

        // get the map
        let mut map = ID_FOR_COLOR.lock().unwrap();

        // add a color for our new source_id
        if !map.contains_key(&source_id) {
            // if source is 0 we use white
            if source_id == 0 {
                map.insert(source_id, (255, 255, 255));
            } else {
                // get a random color
                let len = map.len();
                if len <= COLORS.len() {
                    map.insert(source_id, COLORS[0]);

                    // map.insert(source_id, COLORS[len - 1]);
                } else {
                    map.insert(source_id, (255, 255, 255));
                }
            }
        }

        // get the color
        let Some(col) = map.get(&source_id) else {
            panic!("Failed to get Color for log");
        };

        let l = match severity {
            Severity::Info => format!("{}:{}", "[I]".white().underline(), contents.truecolor(col.0, col.1, col.2)),
            Severity::Warning => format!("{}:{}", "[W]".yellow().underline(), contents.truecolor(col.0, col.1, col.2)),
            Severity::Error => format!("{}:{}", "[E]".red().underline(), contents.truecolor(col.0, col.1, col.2)),
        };

        // write line based on severity
        match severity {
            Severity::Info => println!("{}:{}", "[I]".white().underline(), contents.truecolor(col.0, col.1, col.2)),
            Severity::Warning => println!("{}:{}", "[W]".yellow().underline(), contents.truecolor(col.0, col.1, col.2)),
            Severity::Error => eprintln!("{}:{}", "[E]".red().underline(), contents.truecolor(col.0, col.1, col.2)),
        }
        self.log_buffer.push((severity, l));
    }
}
