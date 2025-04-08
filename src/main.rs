use std::{env, path::Path, process::exit};

pub use app::App;

pub mod app;
mod logging;
pub mod snapshots;
pub mod structures;
pub mod utils;
pub mod var_index;

fn parse_args() -> (String, usize, usize) {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Missing vcd file argument!");
        eprintln!(
            "Usage: {} <path to vcd file> [start clock cycle] [debugging length]",
            args[0]
        );
        exit(1)
    }

    let mut start_clock = 0;
    if let Some(potential_arg) = args.get(2) {
        start_clock = potential_arg.parse::<usize>().unwrap_or_else(|err| {
            eprintln!("Start clock cycle wasn't able to be parsed to usize!");
            eprintln!("Error: {err}");
            eprintln!(
                "Usage: {} <path to vcd file> [start clock cycle] [debugging length]",
                args[0]
            );
            exit(1)
        })
    }

    let mut debugging_length = usize::MAX;
    if let Some(potential_arg) = args.get(3) {
        debugging_length = potential_arg.parse::<usize>().unwrap_or_else(|err| {
            eprintln!("Debugging length wasn't able to be parsed to usize!");
            eprintln!("Error: {err}");
            eprintln!(
                "Usage: {} <path to vcd file> [start clock cycle] [debugging length]",
                args[0]
            );
            exit(1)
        })
    }

    let mut name = args[1].clone();
    if !name.ends_with(".vcd") {
        name += ".vcd";
    }
    let build_name = format!("build/{name}");

    if !Path::new(&name).exists() && !Path::new(&build_name).exists() {
        eprintln!("Couldn't find {name} or {build_name}!");
        eprintln!("Usage: {} <path to vcd file>", args[0]);
        exit(1)
    }

    let correct_name = if Path::new(&name).exists() {
        name
    } else {
        build_name
    };

    (correct_name, start_clock, debugging_length)
}

// fn main() {
//     let (filename, start_clock, debugging_length) = parse_args();
//     dbg!(filename);
//     dbg!(start_clock);
//     dbg!(debugging_length);
// }

fn main() -> color_eyre::Result<()> {
    let (filename, start_clock, debugging_length) = parse_args();

    logging::initialize_logging()?;
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = App::new(&filename, start_clock, debugging_length);
    let result = app.run(terminal);
    ratatui::restore();
    result
}
