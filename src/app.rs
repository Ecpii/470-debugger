use std::cmp::min;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Clear, List, ListState, Paragraph},
    DefaultTerminal, Frame,
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{snapshots::Snapshots, structures::Structures, trace_dbg};

pub struct App {
    /// Is the application running?
    running: bool,
    snapshots: Snapshots,
    watch_list: Vec<String>,
    show_popup: bool,
    search_input: Input,
    search_query: String,
    search_list_state: ListState,
    search_matches: Vec<String>,
    structures: Structures,
    cycle_jump: usize,
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(filename: &str) -> Self {
        trace_dbg!("start");
        let snapshots = Snapshots::new(filename).unwrap();
        let structures = Structures::new(&snapshots);
        Self {
            running: false,
            snapshots,
            watch_list: Vec::new(),
            show_popup: false,
            search_input: Input::default(),
            search_query: String::new(),
            search_list_state: ListState::default(),
            search_matches: Vec::new(),
            structures,
            cycle_jump: 1,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {
        let title = Line::from("o3o Debugger").bold().blue().centered();
        let snapshot = self.snapshots.get().unwrap();
        let mut text = format!(
            "Current Clock Cycle: {}  - Current Time: {}\n",
            snapshot.clock_count, snapshot.time
        );

        for name in self.watch_list.iter() {
            if let Some(value) = self.snapshots.get_var(name) {
                text.push_str(&format!("{}: {}\n", name, value));
            } else {
                text.push_str(&format!("{name} not found!\n"));
            }
        }

        let instructions = Line::from(vec![
            " Watch variable ".into(),
            "</>".blue().bold(),
            format!(" Back {} timesteps ", self.cycle_jump).into(),
            "<Left>".blue().bold(),
            format!(" Forward {} timestep ", self.cycle_jump).into(),
            "<Right>".blue().bold(),
            " Change increment ".into(),
            "<Up/Down>".blue().bold(),
            " Go To Start ".into(),
            "<s>".blue().bold(),
            " Go To End ".into(),
            "<e>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ])
        .centered();

        let block = Block::bordered().title(title).title_bottom(instructions);
        let [top_half, bottom_half] = Layout::vertical([
            Constraint::Length((2 + self.watch_list.len()) as u16),
            Constraint::Fill(1),
        ])
        .areas(block.inner(frame.area()));

        frame.render_widget(block, frame.area());

        frame.render_widget(Paragraph::new(text).centered(), top_half);
        frame.render_stateful_widget(self.structures.clone(), bottom_half, &mut self.snapshots);

        if self.show_popup {
            let block = Block::bordered().title("Watch Variable...");
            let search = Line::from(self.search_input.value());
            let list = List::new(self.search_matches.clone())
                .highlight_style(Style::new().bg(Color::Blue));

            let area = popup_area(frame.area(), 60, 20);
            let vertical = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
            let inner_area = block.inner(area);
            let [search_area, match_area] = vertical.areas(inner_area);

            frame.render_widget(Clear, area); //this clears out the background
            frame.render_widget(block, area);
            frame.render_widget(search, search_area);
            frame.render_stateful_widget(list, match_area, &mut self.search_list_state);
            frame.set_cursor_position((
                area.x + (self.search_input.visual_cursor()) as u16 + 1,
                area.y + 1,
            ));
        }
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        if self.show_popup {
            match (key.modifiers, key.code) {
                (_, KeyCode::Esc | KeyCode::Char('/')) => {
                    self.show_popup = false;
                }
                (_, KeyCode::Enter) => {
                    let value = self.search_input.value().trim();
                    if !value.is_empty() {
                        self.watch_list.push(value.to_owned());
                    }
                    self.search_input.reset();
                    self.show_popup = false;
                }
                (_, KeyCode::Up) => {
                    if let Some(selected) = self.search_list_state.selected() {
                        if selected == 0 {
                            self.search_list_state.select(None);
                            self.search_input = Input::new(self.search_query.clone());
                        } else {
                            self.search_list_state.select(Some(selected - 1));
                            self.search_input =
                                Input::new(self.search_matches[selected - 1].clone());
                        }
                    }
                }
                (_, KeyCode::Down) => {
                    self.search_list_state.select_next();
                    let selected = self.search_list_state.selected().unwrap();
                    let index = min(selected, self.search_matches.len() - 1);
                    self.search_input = Input::new(self.search_matches[index].clone());
                }
                // (_, KeyCode::Char(_)) => {
                // }
                _ => {
                    self.search_input.handle_event(&Event::Key(key));
                    self.search_query = self.search_input.value().to_owned();
                    self.search_matches =
                        self.snapshots.autocomplete_var(self.search_input.value());
                }
            }
            return;
        }

        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            (_, KeyCode::Left) => self.handle_left_key(),
            (_, KeyCode::Right) => self.handle_right_key(),
            (_, KeyCode::Up) => self.handle_up_key(),
            (_, KeyCode::Down) => self.handle_down_key(),
            (_, KeyCode::Char('s')) => self.snapshots.go_to_start(),
            (_, KeyCode::Char('e')) => self.snapshots.go_to_end(),

            (_, KeyCode::Char('/')) => self.show_popup = !self.show_popup,

            // vim bindings
            (_, KeyCode::Char('h')) => self.handle_left_key(),
            (_, KeyCode::Char('k')) => self.handle_up_key(),
            (_, KeyCode::Char('j')) => self.handle_down_key(),
            (_, KeyCode::Char('l')) => self.handle_right_key(),
            _ => {}
        }
    }

    fn handle_left_key(&mut self) {
        self.snapshots.retreat_n(self.cycle_jump);
    }

    fn handle_right_key(&mut self) {
        self.snapshots.advance_n(self.cycle_jump);
    }

    fn handle_up_key(&mut self) {
        self.cycle_jump *= 10;
    }

    fn handle_down_key(&mut self) {
        if self.cycle_jump >= 10 {
            self.cycle_jump /= 10;
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
