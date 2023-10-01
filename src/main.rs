use clap::Parser;
use color_eyre::eyre::{eyre, Context, Result};
use srt_parser::SubRipFile;
use std::io::Write;
use std::{
    fs::{self, OpenOptions},
    ops::Add,
    path::PathBuf,
};
use time::{Duration, Time};

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let original_path = PathBuf::from(&args.subtitle_path);
    let duplicate_path = PathBuf::from(format!("{}.old", &args.subtitle_path));

    fs::rename(&original_path, &duplicate_path)
        .context("Unable to create a copy of the subtitle file")?;

    let subs = SubRipFile::new(duplicate_path)
        .map_err(|e| eyre!(e))
        .context("Unable to parse the subtitle file")?;
    let subs_len = subs.subtitles().len();

    std::fs::File::create(&original_path).context("Unable to create the subtitle file")?;
    let mut file = OpenOptions::new()
        .append(true)
        .open(&original_path)
        .context("Unable to open the created subtitle file")?;

    let offest_seconds = Duration::nanoseconds((args.time_shift * 1_000_000_000.0) as i64);

    for (index, sub) in subs.subtitles().iter().enumerate() {
        let start = sub.start();
        let start_offset = start.add(offest_seconds);
        let start_offset = format_time(start_offset);

        let end = sub.end();
        let end_offset = end.add(offest_seconds);
        let end_offset = format_time(end_offset);

        let sequence_number = sub.sequence_number();
        let text = sub.text();

        if index < subs_len - 1 {
            writeln!(
                file,
                "{sequence_number}\n{start_offset} --> {end_offset}\n{text}\n"
            )
            .context("Unable to write the subtitle file")?;
        } else {
            write!(
                file,
                "{sequence_number}\n{start_offset} --> {end_offset}\n{text}"
            )
            .context("Unable to write the subtitle file")?;
        }
    }

    println!("Done.");

    Ok(())
}

fn format_time(time: Time) -> String {
    format!(
        "{:02}:{:02}:{:02},{:03}",
        time.hour(),
        time.minute() % 60,
        time.second() % 60,
        time.millisecond() % 1000
    )
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Subtitle path
    subtitle_path: String,

    /// Time shift
    #[clap(allow_hyphen_values = true)]
    time_shift: f64,
}
