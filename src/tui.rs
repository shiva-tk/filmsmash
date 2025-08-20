use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Stylize, Style},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Widget, Wrap},
    DefaultTerminal, Frame,
};
use std::io;
use time::Date;

use crate::ranker::Ranker;
use crate::film::Film;

enum Selection {
    Left,
    Right,
    None
}

pub struct Tui {
    ranker: Box<dyn Ranker<Film>>,
    selected: Selection,
    exit: bool
}

impl Tui {
    pub fn new(ranker: Box<dyn Ranker<Film>>) -> Self {
        Tui {ranker, selected: Selection::None, exit: false}
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit && !self.ranker.is_ranked() {
            terminal.draw(|frame| self.draw(frame))?;
            match self.selected {
                Selection::Left => {
                    // Briefly flash highlighted frame by sleeping
                    std::thread::sleep(std::time::Duration::from_millis(150));
                    // Commit selection
                    self.ranker.gt();
                    // Reset selection
                    self.selected = Selection::None;
                },
                Selection::Right => {
                    // Briefly flash highlighted frame by sleeping
                    std::thread::sleep(std::time::Duration::from_millis(150));
                    // Commit selection
                    self.ranker.lt();
                    // Reset selection
                    self.selected = Selection::None;
                },
                Selection::None => {
                    // Handle further key presses
                    self.handle_events()?;
                }
            }
        }

        Ok(())
    }

    pub fn print_top_10(&self) {
        self.ranker.print_top_10();
    }

    pub fn write_ranking(&self) {
        self.ranker.write_ranking();
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent){
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.select_left(),
            KeyCode::Right => self.select_right(),
            KeyCode::Char('h') => self.select_left(),
            KeyCode::Char('l') => self.select_right(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn select_left(&mut self) {
        self.selected = Selection::Left;
    }

    fn select_right(&mut self) {
        self.selected = Selection::Right;
    }
}

impl Widget for &mut Tui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Outer block with app title
        let app_title = Line::from(" filmsmash ".bold());
        let quit_instruction = Line::from(vec![
            "Quit ".into(),
            "<q>".magenta().bold()
        ]);
        let outer_block = Block::new()
            .borders(Borders::TOP)
            .border_type(BorderType::Double)
            .title(app_title.centered())
            .title_bottom(quit_instruction.centered());

        // Split the area horizontally into two equal parts
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(outer_block.inner(area));

        // Left film + instructions
        let left_instruction = Line::from(vec![
            " ".into(),
            "<Left>".magenta().bold(),
            " or ".into(),
            "<h>".magenta().bold(),
            " to select ".into(),
        ]);
        let mut left_block = Block::bordered()
            .title_bottom(left_instruction.left_aligned())
            .border_set(border::ROUNDED);

        if matches!(self.selected, Selection::Left) {
            left_block = left_block.border_style(Style::default().red().bold());
        }

        if let Some(left_film) = self.ranker.left() {
            let view = FilmView::from(left_film);
            Paragraph::new(Text::from(&view))
                .alignment(Alignment::Center)
                .block(left_block)
                .wrap(Wrap { trim: true })
                .render(chunks[0], buf);
        } else {
            Paragraph::new("No film")
                .alignment(Alignment::Center)
                .block(left_block)
                .render(chunks[0], buf);
        }

        // Right film + instructions
        let right_instruction = Line::from(vec![
            " ".into(),
            "<Right>".magenta().bold(),
            " or ".into(),
            "<l>".magenta().bold(),
            " to select ".into(),
        ]);
        let mut right_block = Block::bordered()
            .title_bottom(right_instruction.right_aligned())
            .border_set(border::ROUNDED);

        if matches!(self.selected, Selection::Right) {
            right_block = right_block.border_style(Style::default().red().bold());
        }

        if let Some(right_film) = self.ranker.right() {
            let view = FilmView::from(right_film);
            Paragraph::new(Text::from(&view))
                .alignment(Alignment::Center)
                .block(right_block)
                .wrap(Wrap { trim: true })
                .render(chunks[1], buf);
        } else {
            Paragraph::new("No film")
                .alignment(Alignment::Center)
                .block(right_block)
                .render(chunks[1], buf);
        }

        // Finally, render the outer block border
        outer_block.render(area, buf);
    }
}

struct FilmView {
    name: String,
    year: u32,
    genre: String,
    director: String,
    plot: String,
    date_watched: Date
}

impl From<&mut Film> for FilmView {
    fn from(film: &mut Film) -> Self {
        FilmView {
            name: film.name().to_string(),
            year: film.year(),
            genre: film.genre().unwrap_or("Unknown Genre").to_string(),
            director: film.director().unwrap_or("Unknown").to_string(),
            plot: film.plot().unwrap_or("No plot available").to_string(),
            date_watched: film.date_watched(),
        }
    }
}

impl<'a> From<&'a FilmView> for Text<'a> {
    fn from(view: &'a FilmView) -> Self {
        Text::from(vec![
            Line::from(view.name.as_str().bold()),
            Line::from(format!("Directed by {}", view.director)),
            Line::from(format!("{} · {}", view.year, view.genre).italic()),
            Line::from(view.plot.as_str().gray()),
            Line::from(format!("Watched on {}", view.date_watched)),
        ])
    }
}
