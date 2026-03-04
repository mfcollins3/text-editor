use crossterm::event::Event;
use ratatui::{DefaultTerminal, Frame};

mod app;
mod editor;
mod ui;

use app::App;

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame: &mut Frame| app.draw(frame))?;

        if let Event::Key(key) = crossterm::event::read()? {
            if app.handle_key(key).should_quit() {
                return Ok(());
            }
        }
    }
}
