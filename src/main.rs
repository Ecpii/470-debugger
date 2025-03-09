use std::{env, path::Path, process::exit};

pub use app::App;

pub mod app;
mod logging;
pub mod snapshots;
pub mod structures;
pub mod var_index;

fn parse_args() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Missing vcd file argument!");
        eprintln!("Usage: {} <path to vcd file>", args[0]);
        exit(1)
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

    if Path::new(&name).exists() {
        name
    } else {
        build_name
    }
}

fn main() -> color_eyre::Result<()> {
    let filename = parse_args();
    logging::initialize_logging()?;
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = App::new(&filename);
    let result = app.run(terminal);
    ratatui::restore();
    result
}
