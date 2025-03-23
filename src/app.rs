use std::cmp::min;

use crate::utils::{load_watch_list, save_watch_list, DisplayType};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Clear, List, ListState, Paragraph},
    DefaultTerminal, Frame,
};
use tui_input::{backend::crossterm::EventHandler, Input};
use vcd::ScopeItem;

use crate::{snapshots::Snapshots, structures::Structures, trace_dbg};

#[derive(Clone, Copy, Debug)]
enum PopupType {
    Search,
    Help,
}

pub struct App {
    /// Is the application running?
    running: bool,
    snapshots: Snapshots,
    watch_list: Vec<(String, DisplayType)>,
    show_popup: Option<PopupType>,
    search_input: Input,
    search_query: String,
    search_list_state: ListState,
    search_matches: Vec<String>,
    structures: Structures,
    cycle_jump: usize,
    watch_list_state: ListState,
    error_message: Option<String>,
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
        let search_query = snapshots.get_base() + ".";
        let search_matches = snapshots.autocomplete_var(&search_query);

        let mut s = Self {
            running: false,
            snapshots,
            watch_list: Vec::new(),
            show_popup: None,
            search_input: Input::new(search_query.clone()),
            search_query,
            search_list_state: ListState::default(),
            search_matches,
            structures,
            cycle_jump: 1,
            watch_list_state: ListState::default(),
            error_message: None,
        };

        s.load_watch_list();

        s
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let _ = terminal.clear();
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    fn render_watch_list(&mut self, frame: &mut Frame, area: Rect) {
        let snapshot = self.snapshots.get().unwrap();
        let [first_line, rest] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
        let time_marker = format!(
            "Current Clock Cycle: {}  - Current Time: {}\n",
            snapshot.clock_count, snapshot.time
        );

        let mut lines = Vec::new();

        for (name, display_type) in self.watch_list.iter() {
            if let Some(value) = self.snapshots.get_var(name) {
                lines.push(
                    Text::from(format!("{}: {}\n", name, value.format(display_type))).centered(),
                );
            } else {
                lines.push(Text::from(format!("{name} not found!\n")).dim().centered());
            }
        }

        let list = List::new(lines).highlight_style(Style::new().on_blue());

        frame.render_widget(Line::from(time_marker).centered(), first_line);
        frame.render_stateful_widget(list, rest, &mut self.watch_list_state);
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {
        let title = if let Some(error) = &self.error_message {
            Line::from(error.clone()).bold().red().centered()
        } else {
            Line::from("o3o Debugger").bold().blue().centered()
        };

        let instructions = if self.watch_list_state.selected().is_some() {
            Line::from(vec![
                " Help ".into(),
                "<?>".blue().bold(),
                " Watch variable ".into(),
                "</>".blue().bold(),
                " Delete ".into(),
                "<d>".blue().bold(),
                " Change type ".into(),
                "<c>".blue().bold(),
                " Quit ".into(),
                "<Q> ".blue().bold(),
            ])
        } else {
            Line::from(vec![
                " Help ".into(),
                "<?>".blue().bold(),
                " Watch variable ".into(),
                "</>".blue().bold(),
                format!(" Back {} timesteps ", self.cycle_jump).into(),
                "<Left>".blue().bold(),
                format!(" Forward {} timesteps ", self.cycle_jump).into(),
                "<Right>".blue().bold(),
                " Change increment ".into(),
                "<+/->".blue().bold(),
                " Quit ".into(),
                "<Q> ".blue().bold(),
            ])
        }
        .centered();

        let block = Block::bordered().title(title).title_bottom(instructions);
        let [top_half, bottom_half] = Layout::vertical([
            Constraint::Length((2 + self.watch_list.len()) as u16),
            Constraint::Fill(1),
        ])
        .areas(block.inner(frame.area()));
        frame.render_widget(block, frame.area());

        // render both halves (watch list and structures)
        self.render_watch_list(frame, top_half);
        frame.render_stateful_widget(self.structures.clone(), bottom_half, &mut self.snapshots);

        if let Some(popup_type) = self.show_popup {
            let area = popup_area(frame.area(), 60, 20);

            frame.render_widget(Clear, area); //this clears out the background

            match popup_type {
                PopupType::Search => {
                    let block = Block::bordered().title("Watch Variable...");
                    let search = Line::from(self.search_input.value());
                    let list = List::new(self.search_matches.clone())
                        .highlight_style(Style::new().bg(Color::Blue));

                    let vertical = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
                    let inner_area = block.inner(area);
                    let [search_area, match_area] = vertical.areas(inner_area);

                    frame.render_widget(block, area);
                    frame.render_widget(search, search_area);
                    frame.render_stateful_widget(list, match_area, &mut self.search_list_state);
                    frame.set_cursor_position((
                        area.x + (self.search_input.visual_cursor()) as u16 + 1,
                        area.y + 1,
                    ));
                }
                PopupType::Help => {
                    let instructions = Paragraph::new(vec![
                        Line::from(vec!["<?>".blue().bold(), " Help\n".into()]),
                        Line::from(vec![
                            "</>".blue().bold(),
                            " Add variable to watch list\n".into(),
                        ]),
                        Line::from(vec![
                            "<Left>".blue().bold(),
                            format!(" Back {} timesteps / ", self.cycle_jump).into(),
                            "<Right>".blue().bold(),
                            format!(" Forward {} timesteps\n", self.cycle_jump).into(),
                        ]),
                        Line::from(vec!["<+/->".blue().bold(), " Change increment\n".into()]),
                        Line::from(vec!["<Up/Down>".blue().bold(), " Select Variable\n".into()]),
                        Line::from(vec!["<d>".blue().bold(), " Delete selected watch\n".into()]),
                        Line::from(vec![
                            "<c>".blue().bold(),
                            " Change selected watch type (binary, decimal, hex)\n".into(),
                        ]),
                        Line::from(vec![
                            "<s>".blue().bold(),
                            " Go To Start / ".into(),
                            "<e>".blue().bold(),
                            " Go To End\n".into(),
                        ]),
                        Line::from(vec!["<q>".blue().bold(), " Quit ".into()]),
                    ])
                    .block(Block::bordered().title("Keybindings"));

                    frame.render_widget(instructions, area);
                }
            }
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
        self.error_message = None;
        if matches!(self.show_popup, Some(PopupType::Search)) {
            match (key.modifiers, key.code) {
                (_, KeyCode::Esc | KeyCode::Char('/')) => {
                    self.show_popup = None;
                }
                (_, KeyCode::Enter) => {
                    self.handle_search_enter();
                }
                (_, KeyCode::Up | KeyCode::BackTab) => {
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
                (_, KeyCode::Down | KeyCode::Tab) => {
                    self.search_list_state.select_next();
                    let selected = self.search_list_state.selected().unwrap();
                    let index = min(selected, self.search_matches.len() - 1);
                    self.search_input = Input::new(self.search_matches[index].clone());
                }
                _ => {
                    if self.search_input.handle_event(&Event::Key(key)).is_some() {
                        self.search_query = self.search_input.value().to_owned();
                        self.search_matches =
                            self.snapshots.autocomplete_var(self.search_input.value());
                        self.search_list_state.select(None);
                    }
                }
            }
            return;
        }

        match (key.modifiers, key.code) {
            (_, KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            (_, KeyCode::Char('=') | KeyCode::Char('+')) => self.increase_jump(),
            (_, KeyCode::Char('-') | KeyCode::Char('_')) => self.decrease_jump(),
            (_, KeyCode::Char('s') | KeyCode::Char('0')) => self.snapshots.go_to_start(),
            (_, KeyCode::Char('e') | KeyCode::Char('$')) => self.snapshots.go_to_end(),
            (_, KeyCode::Char('d')) => self.delete_selected_watch(),

            // (_, KeyCode::Char('w')) => self.save_watch_list(),
            // (_, KeyCode::Char('r')) => {
            //     self.load_watch_list();
            // }
            (_, KeyCode::Char('c')) => self.change_selected_watch_mode(),

            (_, KeyCode::Esc) => {
                if self.show_popup.is_some() {
                    self.show_popup = None;
                }
            }
            (_, KeyCode::Char('?')) => {
                if self.show_popup.is_some() {
                    self.show_popup = None;
                } else {
                    self.show_popup = Some(PopupType::Help)
                }
            }
            (_, KeyCode::Char('/')) => self.show_popup = Some(PopupType::Search),

            // vim bindings
            (_, KeyCode::Left | KeyCode::Char('h')) => self.handle_left_key(),
            (_, KeyCode::Down | KeyCode::Char('j')) => self.handle_down_key(),
            (_, KeyCode::Up | KeyCode::Char('k')) => self.handle_up_key(),
            (_, KeyCode::Right | KeyCode::Char('l')) => self.handle_right_key(),
            _ => {
                self.structures.on_key_event(key);
            }
        }
    }

    fn save_watch_list(&mut self) {
        if self.watch_list.is_empty() {
            return;
        }

        if let Err(e) = save_watch_list(&self.watch_list, "last") {
            self.error_message = Some(format!("Error saving watch list: {}", e));
        };
    }

    fn load_watch_list(&mut self) {
        // todo: we could change the name so you can have multiple watch lists
        match load_watch_list("last") {
            Ok(list) => {
                self.watch_list = list;
            }
            Err(e) => {
                self.error_message = Some(format!("Error loading watch list: {}", e));
            }
        }
    }

    fn handle_search_enter(&mut self) {
        let value = if let Some(index) = self.search_list_state.selected() {
            &self.search_matches[index]
        } else {
            self.search_input.value().trim()
        };

        if !value.is_empty() {
            if self.snapshots.get_var(value).is_some() {
                self.watch_list
                    .push((value.to_owned(), DisplayType::Binary));
            } else if let Some(scope) = self.snapshots.get_scope(value) {
                for s in scope.items.iter() {
                    if let ScopeItem::Var(v) = s {
                        let new_name = value.to_owned() + "." + &v.reference;
                        self.watch_list.push((new_name, DisplayType::Binary));
                    }
                }
            } else {
                self.error_message = Some(String::from("Invalid wire path!"));
                return;
            }
        }
        self.search_input = Input::new(self.snapshots.get_base() + ".");
        self.show_popup = None;
    }

    fn change_selected_watch_mode(&mut self) {
        let Some(index) = self.watch_list_state.selected() else {
            return;
        };
        let current_type = self.watch_list.get(index).unwrap().1;
        self.watch_list[index].1 = current_type.next();
    }

    fn delete_selected_watch(&mut self) {
        let Some(index) = self.watch_list_state.selected() else {
            return;
        };
        self.watch_list.remove(index);
    }

    fn handle_left_key(&mut self) {
        self.snapshots.retreat_n(self.cycle_jump);
    }

    fn handle_right_key(&mut self) {
        self.snapshots.advance_n(self.cycle_jump);
    }

    fn handle_up_key(&mut self) {
        if let Some(0) = self.watch_list_state.selected() {
            self.watch_list_state.select(None)
        } else if self.watch_list_state.selected().is_some() {
            self.watch_list_state.select_previous();
        }
    }

    fn handle_down_key(&mut self) {
        if !self.watch_list.is_empty() {
            self.watch_list_state.select_next();
        }
    }

    fn increase_jump(&mut self) {
        self.cycle_jump *= 10;
    }

    fn decrease_jump(&mut self) {
        if self.cycle_jump >= 10 {
            self.cycle_jump /= 10;
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.save_watch_list();
        self.running = false;
    }
}
