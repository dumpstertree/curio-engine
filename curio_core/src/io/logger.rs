use crate::Severity;
use colored::Colorize;
use std::collections::HashMap;

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

/// An object for formating, displaying and storing logs
#[repr(C)]
pub struct Logger {
    log_buffer: Vec<(Severity, String)>,
    id_for_color: HashMap<i32, (u8, u8, u8)>,
    log_level: Severity,
}
impl Logger {
    /// Create a logger with desired log level
    pub fn new(log_level: Severity) -> Logger {
        Logger {
            log_buffer: Vec::new(),
            id_for_color: HashMap::new(),
            log_level,
        }
    }
    // Get any logs that have been made since the last drain. Used for editor displays or log dumping.
    pub fn drain(&mut self) -> Vec<(Severity, String)> {
        let buf = self.log_buffer.to_vec();
        self.log_buffer.clear();
        buf
    }

    /// Log a message. source_id corresponds to the owner and severity represents the log level
    pub fn log(&mut self, source_id: i32, severity: Severity, contents: &str) {
        // if severity is too low we ignore
        if severity < self.log_level {
            return;
        }

        // add a color for our new source_id
        if !self.id_for_color.contains_key(&source_id) {
            // if source is 0 we use white
            if source_id == 0 {
                self.id_for_color.insert(source_id, (255, 255, 255));
            } else {
                // get a random color
                let len = self.id_for_color.len();
                if len <= COLORS.len() {
                    self.id_for_color.insert(source_id, COLORS[0]);
                    // map.insert(source_id, COLORS[len - 1]);
                } else {
                    self.id_for_color.insert(source_id, (255, 255, 255));
                }
            }
        }

        // get the color
        let Some(col) = self.id_for_color.get(&source_id) else {
            panic!("Failed to get Color for log");
        };

        // create an entry
        let log_entry = match severity {
            Severity::Info => format!("{}:{}", "[I]".white().underline(), contents.truecolor(col.0, col.1, col.2)),
            Severity::Warning => format!("{}:{}", "[W]".yellow().underline(), contents.truecolor(col.0, col.1, col.2)),
            Severity::Error => format!("{}:{}", "[E]".red().underline(), contents.truecolor(col.0, col.1, col.2)),
        };

        // add it to the buffer
        self.log_buffer.push((severity.clone(), log_entry));

        // write line to console
        match severity {
            Severity::Info => println!("{}:{}", "[I]".white().underline(), contents.truecolor(col.0, col.1, col.2)),
            Severity::Warning => println!("{}:{}", "[W]".yellow().underline(), contents.truecolor(col.0, col.1, col.2)),
            Severity::Error => eprintln!("{}:{}", "[E]".red().underline(), contents.truecolor(col.0, col.1, col.2)),
        }
    }
}
unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}
