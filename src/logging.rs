use std::fs::File;
use std::path::Path;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::fmt::writer::MakeWriterExt;

use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Seek, Write};

const MAX_LOG_LINES: usize = 250;

fn prepare_log_file(path: &Path) -> io::Result<File> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    let reader = BufReader::new(&file);
    let lines: Vec<String> = reader.lines().filter_map(|line| line.ok()).collect();

    if lines.len() > MAX_LOG_LINES {
        let last_lines = lines
            .into_iter()
            .rev()
            .take(MAX_LOG_LINES)
            .rev()
            .collect::<Vec<_>>();

        file.set_len(0)?;
        file.seek(io::SeekFrom::Start(0))?;

        for line in last_lines {
            writeln!(file, "{}", line)?;
        }

        OpenOptions::new().append(true).open(path)
    } else {
        OpenOptions::new().append(true).open(path)
    }
}

pub fn init() {
    let log_file =
        prepare_log_file(Path::new("vrc-manager.log")).expect("Failed to prepare log file");

    let combined_writer = std::io::stdout
        .with_max_level(Level::INFO)
        .and(log_file.with_max_level(Level::INFO));

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(combined_writer)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
