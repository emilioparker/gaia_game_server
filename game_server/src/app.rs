use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers}, terminal};
use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame,
};

use crate::AppData;

pub struct App {
    /// Is the application running?
    /// 
    pub running: bool,
    counter: u32,
    app_data: AppData,
}

pub struct AppError{

}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(data: AppData) -> Self 
    {
        Self { running: true, counter:0, app_data: data}
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), AppError> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.draw(frame));
            self.handle_crossterm_events();
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {
        let title = Line::from(format!("Ratatui Simple Template:{}", self.counter))
            .bold()
            .blue()
            .centered();
        let text = "Hello, Ratatui!\n\n\
            Created using https://github.com/ratatui/templates\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.";
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        )
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<(), AppError> {
        if let Ok(read_result) = event::read()
        {
            match read_result
            {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
