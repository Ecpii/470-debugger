pub use app::App;

pub mod app;
mod logging;
pub mod rob;
pub mod rs;
pub mod snapshots;
pub mod var_index;

fn main() -> color_eyre::Result<()> {
    logging::initialize_logging()?;
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
