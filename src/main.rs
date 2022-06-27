mod enums;
mod tasks_layout;
mod group_layout;

use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::widgets::BorderType;
use crate::enums::FocusedLayout;

struct App {
    focused_layout: FocusedLayout,
    last_layout: FocusedLayout,
}

impl App {
    fn new() -> App {
        App {
            focused_layout: FocusedLayout::GroupsLayout,
            last_layout: FocusedLayout::None
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('p') => {
                    match app.focused_layout {
                        FocusedLayout::OptionsLayout => {
                            app.focused_layout = app.last_layout;
                            app.last_layout = FocusedLayout::OptionsLayout;
                        },
                        _ => {
                            app.last_layout = app.focused_layout;
                            app.focused_layout = FocusedLayout::OptionsLayout;
                        }
                    }
                },
                KeyCode::Left => {
                    app.last_layout = app.focused_layout;
                    app.focused_layout = FocusedLayout::GroupsLayout
                },
                KeyCode::Right => {
                    app.last_layout = app.focused_layout;
                    app.focused_layout = FocusedLayout::TasksLayout
                },
                KeyCode::Char('q') => return Ok(()),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();

    let chunks = Layout::default()
        .constraints([Constraint::Percentage(10), Constraint::Percentage(80)].as_ref())
        .split(size);

    let top_block = Block::default()
        .borders(Borders::ALL);

    let title = Paragraph::new("ToDo List version 0.1!").block(top_block).alignment(Alignment::Center);

    let lower_chunks = Layout::default()
        .constraints([Constraint::Percentage(20), Constraint::Percentage(70)].as_ref())
        .direction(Direction::Horizontal)
        .split(chunks[1]);

    let mut groups_block = Block::default()
        .title("Groups")
        .borders(Borders::ALL)
        .style(Style::default());

    let mut tasks_block = Block::default()
        .title("Tasks")
        .borders(Borders::ALL)
        .style(Style::default());

    let mut options_block = Block::default().title("Options").borders(Borders::ALL);
    let area = centered_rect(40, 20, lower_chunks[1]);

    match app.focused_layout {
        FocusedLayout::GroupsLayout => groups_block = groups_block.style(Style::default().add_modifier(Modifier::BOLD))
                                      .border_type(BorderType::Thick),
        FocusedLayout::TasksLayout => tasks_block = tasks_block.style(Style::default().add_modifier(Modifier::BOLD))
                                      .border_type(BorderType::Thick),
        _ => options_block = options_block.style(Style::default().add_modifier(Modifier::BOLD))
            .border_type(BorderType::Thick),
    };

    f.render_widget(title, chunks[0]);
    f.render_widget(groups_block, lower_chunks[0]);
    f.render_widget(tasks_block, lower_chunks[1]);

    match app.focused_layout {
        FocusedLayout::OptionsLayout => {
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(options_block, area);
        },
        _ => {}
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
