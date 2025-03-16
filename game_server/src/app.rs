use std::{env::consts, time::Duration};

use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers}, terminal};
use ratatui::{
    layout::{Constraint, Direction, Layout}, style::{Color, Style, Stylize}, symbols::{block, border}, text::Line, widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph, Sparkline, Widget}, DefaultTerminal, Frame
};

use crate::AppData;

pub struct App {
    pub running: bool,
    last_check: u64,
    // receive
    last_received_udp_packages:u64,
    received_udp_packages_per_second:f32,
    last_received_bytes:u64,
    received_bytes_per_second:f32,
    // in bytes graph
    received_bytes_graph_data:[u64;64],
    received_bytes_graph_max: u64,
    // sent
    last_sent_udp_packages:u64,
    sent_udp_packages_per_second:f32,
    last_sent_game_packages:u64,
    sent_game_packets_per_second:f32,
    last_sent_bytes:u64,
    sent_bytes_per_second:f32,

    // out bytes graph
    sent_bytes_graph_data:[u64;64],
    sent_bytes_graph_max: u64,

    online_players:u32,
    app_data: AppData,
}

pub struct AppError{

}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(data: AppData) -> Self 
    // pub fn new() -> Self 
    {
        Self { running: true,
            last_check:0,
            last_received_udp_packages:0,
            last_received_bytes:0,
            app_data: data,
            online_players: 0,
            received_udp_packages_per_second: 0f32,
            received_bytes_per_second: 0f32,
            received_bytes_graph_data: [0; 64],
            received_bytes_graph_max: 0,
            last_sent_udp_packages: 0,
            sent_udp_packages_per_second: 0f32,
            last_sent_game_packages: 0,
            sent_game_packets_per_second: 0f32,
            last_sent_bytes: 0,
            sent_bytes_per_second: 0f32,
            sent_bytes_graph_data: [0;64],
            sent_bytes_graph_max: 0,
        }
        // Self { running: true, counter:0}
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), AppError> {
        self.running = true;
        while self.running {
            // cli_log::info!("running!: {}", 0);
            
            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            let time_span = current_time_in_millis - self.last_check;
            if time_span > 1000
            {
                self.last_check = current_time_in_millis;

                // online players
                self.online_players = self.app_data.game_status.online_players.load(std::sync::atomic::Ordering::Relaxed);

                //----------------------------------- IN
                // received packets
                let count = self.app_data.game_status.received_packets.load(std::sync::atomic::Ordering::Relaxed);
                let received_packets_since_last_check = (count - self.last_received_udp_packages) as f32;
                self.last_received_udp_packages = count;
                self.received_udp_packages_per_second = received_packets_since_last_check / (time_span as f32 / 1000f32);

                // received bytes
                let bytes = self.app_data.game_status.received_bytes.load(std::sync::atomic::Ordering::Relaxed);
                let last_received_bytes_since_last_check = bytes - self.last_received_bytes;
                self.last_received_bytes = bytes;
                self.received_bytes_per_second = last_received_bytes_since_last_check as f32 / (time_span as f32 / 1000f32);

                // storing bytes received in circular array
                self.received_bytes_graph_data.rotate_right(1);
                self.received_bytes_graph_data[0] = last_received_bytes_since_last_check;
                let max =  self.received_bytes_graph_data.iter().max().map(|v| *v);
                self.received_bytes_graph_max =  max.unwrap_or(0);

                //----------------------------------- OUT
                // sent packets
                let count = self.app_data.game_status.sent_udp_packets.load(std::sync::atomic::Ordering::Relaxed);
                let sent_udp_packets_since_last_check = (count - self.last_sent_udp_packages) as f32;
                self.last_sent_udp_packages = count;
                self.sent_udp_packages_per_second = sent_udp_packets_since_last_check / (time_span as f32 / 1000f32);

                // sent game packets
                let count = self.app_data.game_status.sent_game_packets.load(std::sync::atomic::Ordering::Relaxed);
                let sent_game_packets_since_last_check = (count - self.last_sent_game_packages) as f32;
                self.last_sent_game_packages = count;
                self.sent_game_packets_per_second = sent_game_packets_since_last_check / (time_span as f32 / 1000f32);

                // sent bytes
                let bytes = self.app_data.game_status.sent_bytes.load(std::sync::atomic::Ordering::Relaxed);
                let last_sent_bytes_since_last_check = bytes - self.last_sent_bytes;
                self.last_sent_bytes = bytes;
                self.sent_bytes_per_second = last_sent_bytes_since_last_check as f32 / (time_span as f32 / 1000f32);

                // storing bytes sent in circular array
                self.sent_bytes_graph_data.rotate_right(1);
                self.sent_bytes_graph_data[0] = last_sent_bytes_since_last_check;
                let max =  self.sent_bytes_graph_data.iter().max().map(|v| *v);
                self.sent_bytes_graph_max =  max.unwrap_or(0);

            }

            terminal.draw(|frame| self.draw(frame));
            // cli_log::info!("running2!: {}", 0);
            if let Ok(poll_result) =  event::poll(Duration::from_millis(100))
            {
                if poll_result
                {
                    self.handle_crossterm_events();
                }
            }
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {

        // let online_players = self.app_data.game_status.online_players.load(std::sync::atomic::Ordering::Relaxed);

        let main_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(vec![
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1)
        ]).split(frame.area());

        let title = Line::from(format!("Gaia Game Server({} players online)", self.online_players))
            .bold()
            .blue()
            .centered();

        let title_block = Paragraph::new(title);
        frame.render_widget(title_block, main_layout[0]);

        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let instructions_block = Paragraph::new(instructions).centered();
        frame.render_widget(instructions_block, main_layout[2]);

        let inner_layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(50), Constraint::Percentage(50)
            ])
            .split(main_layout[1]);

        let input_layout = inner_layout[0];
        let output_layout = inner_layout[1];

        let input_data_title = Line::from(format!("input"))
            .centered();

        let input_data_block = Block::bordered()
                .title(input_data_title.centered())
                .border_set(border::PLAIN);

        let output_data_title = Line::from(format!("output"))
            .centered();
        let output_data_block = Block::bordered()
                .title(output_data_title.centered())
                .border_set(border::PLAIN);

        frame.render_widget(input_data_block, input_layout);
        frame.render_widget(output_data_block, output_layout);

        // input sparkline

        let input_spark_line_title = Line::from(vec![
            "In: ".into(),
            format!("{:.2} UDP p/s", self.received_udp_packages_per_second).blue().bold(),
            "  ".into(),
            Self::format_bytes_per_second(self.received_bytes_per_second).blue().bold(),
            "    ".into(),
            "Max: ".into(),
            Self::format_bytes(self.received_bytes_graph_max).blue().bold(),
        ]);

        let input_inner_layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1), Constraint::Length(5), Constraint::Length(18), Constraint::Min(0)
            ])
            .split(input_layout);

        let input_sparkline = Sparkline::default()
            .block(
                Block::new()
                    .borders(Borders::LEFT | Borders::RIGHT)
                    .title(input_spark_line_title),
            )
            .data(&self.received_bytes_graph_data)
            .style(Style::default().fg(Color::Yellow));
        
        frame.render_widget(input_sparkline, input_inner_layout[1]);
        

        let bar_chart = BarChart::default()
            .block(Block::bordered().title("pipes"))
            .bar_width(1)
            .direction(Direction::Horizontal)
            .bar_style(Style::new().green().on_white())
            .value_style(Style::new().black())
            .label_style(Style::new().white())
            .bar_gap(1)
            .data(&[("gameplay", 0), ("B1", 2), ("B2", 4), ("B3", 3)])
            // .data(BarGroup::default().bars(&[Bar::default().value(10), Bar::default().value(20)]))
            .max(10);


        frame.render_widget(bar_chart, input_inner_layout[2]);

        // end of input sparkline 
        // output sparkline 

        let online_players = self.online_players;
        let output_spark_line_title = Line::from(vec![
            "Out: ".into(),
            format!("{:.2} UDP p/s", self.sent_udp_packages_per_second * online_players as f32).blue().bold(),
            "  ".into(),
            format!("{:.2} Game p/s", self.sent_game_packets_per_second * online_players as f32).blue().bold(),
            "  ".into(),
            Self::format_bytes_per_second(self.sent_bytes_per_second * online_players as f32).blue().bold(),
            "    ".into(),
            "Max: ".into(),
            Self::format_bytes(self.sent_bytes_graph_max * online_players as u64).blue().bold(),
        ]);

        let output_inner_layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1), Constraint::Length(5), Constraint::Length(3),  Constraint::Min(1)
            ])
            .split(output_layout);

        let output_sparkline = Sparkline::default()
            .block(
                Block::new()
                    .borders(Borders::LEFT | Borders::RIGHT)
                    .title(output_spark_line_title),
            )
            .data(&self.sent_bytes_graph_data)
            .style(Style::default().fg(Color::Green));
        
        frame.render_widget(output_sparkline, output_inner_layout[1]);

        // let bottom_output_spark_line_title = Line::from(vec![
        //     "Out Per Player: ".into(),
        //     format!("{:.2} UDP p/s", self.sent_udp_packages_per_second).blue().bold(),
        //     "  ".into(),
        //     format!("{:.2} Game p/s", self.sent_game_packets_per_second).blue().bold(),
        //     "  ".into(),
        //     Self::format_bytes_per_second(self.sent_bytes_per_second).blue().bold()
        // ]);

        // let bottom_block =  Paragraph:: new(bottom_output_spark_line_title).centered().block(Block::new().borders(Borders::TOP | Borders::BOTTOM));
        // frame.render_widget(bottom_block, output_inner_layout[2]);





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

    // /// Create a horizontal bar chart from the temperatures data.
    // fn horizontal_barchart(temperatures: &[u8]) -> BarChart {
    //     let bars: Vec<Bar> = temperatures
    //         .iter()
    //         .enumerate()
    //         .map(|(hour, value)| horizontal_bar(value))
    //         .collect();
    //     let title = Line::from("Weather (Horizontal)").centered();
    //     BarChart::default()
    //         .block(Block::new().title(title))
    //         .data(BarGroup::default().bars(&bars))
    //         .bar_width(1)
    //         .bar_gap(0)
    //         .direction(Direction::Horizontal)
    // }

    // fn horizontal_bar(amount: &u8) -> Bar {
    //     let style = Style::new().fg(Color::Yellow);
    //     Bar::default()
    //         .value(u64::from(*amount))
    //         .label(Line::from(format!("InBarLabel")))
    //         .text_value(format!("{amount:>3}°"))
    //         .style(style)
    //         .value_style(style.reversed())
    // }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }


    fn format_bytes_per_second(data: f32) -> String
    {
        if data > 1024.0 * 1024.0
        {
            let mb = data/(1024.0 *1024.0);
            return format!("{mb:.2} MB/s");
        }
        if data > 1024.0
        {
            let kb = data/1024.0;
            return format!("{kb:.2} KB/s");
        }
        else 
        {
            return format!("{data:.2} B/s");
        }
    }

    fn format_bytes(data: u64) -> String
    {
        if data > 1024 * 1024
        {
            let mb = data as f32 /(1024.0 *1024.0);
            return format!("{mb:.2} MB");
        }
        if data > 1024
        {
            let kb = data as f32/1024.0;
            return format!("{kb:.2} KB");
        }
        else 
        {
            return format!("{data:.2} B");
        }
    }
}
