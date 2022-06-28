mod enums;
mod tasks_layout;
mod group_layout;
mod data_manager;

use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::data_manager::DataManager;
use crate::enums::FocusedLayout;
use crate::group_layout::GroupLayout;
use crate::tasks_layout::{TaskLayout};


trait LayoutState {
    fn is_in_edit_mode(&self) -> bool;
    fn handle_input(&mut self, data_manager: &mut DataManager, key_code: crossterm::event::KeyEvent);
    fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>, frame_size: &Rect);
    fn create_and_render_base_block<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>);
    fn create_and_render_item_list<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>, frame_size: &Rect);
    fn create_and_render_edit_mode<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>);
}


struct App {
    focused_layout: FocusedLayout,
    last_layout: FocusedLayout,
    group_layout: GroupLayout,
    task_layout: TaskLayout,
    run: bool,
    data_manager: DataManager
}

impl App {
    fn new() -> App {
        App {
            focused_layout: FocusedLayout::GroupsLayout,
            last_layout: FocusedLayout::None,
            group_layout: GroupLayout::new(),
            task_layout: TaskLayout::new(),
            run: true,
            data_manager: DataManager::new()
        }
    }

    pub fn update_state(&mut self, new_focused_layout: FocusedLayout) {
        self.last_layout = self.focused_layout;
        self.focused_layout = new_focused_layout;
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
    let mut app = App::new();
    app.data_manager.load_state();
    let res = run_app(&mut terminal, &mut app);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {

    while app.run {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {

            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                if !app.group_layout.is_in_edit_mode() && !app.task_layout.is_in_edit_mode() {
                    return Ok(());
                }
            } else if key.code == KeyCode::Left {
                app.update_state(FocusedLayout::GroupsLayout);
                app.data_manager.selected_task = 0;
            } else if key.code == KeyCode::Right {
                app.update_state(FocusedLayout::TasksLayout);
            }

            match app.focused_layout {
                FocusedLayout::GroupsLayout => {
                    app.group_layout.handle_input(&mut app.data_manager, key);
                },
                FocusedLayout::TasksLayout => {
                    app.task_layout.handle_input(&mut app.data_manager,key);
                },
                _ => {}
            }
        }
    }

    return Ok(());
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    let chunks = Layout::default()
        .constraints([Constraint::Percentage(10), Constraint::Percentage(80)].as_ref())
        .split(size);

    let top_block = Block::default()
        .borders(Borders::ALL);

    let title = Paragraph::new("Term-do 0.5").block(top_block).alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    let lower_chunks = Layout::default()
        .constraints([Constraint::Percentage(20), Constraint::Percentage(70)].as_ref())
        .direction(Direction::Horizontal)
        .split(chunks[1]);

    <GroupLayout as LayoutState>::ui(f, app, &lower_chunks, &f.size());
    <TaskLayout as LayoutState>::ui(f, app, &lower_chunks, &f.size());

    //
    // match app.focused_layout {
    //     FocusedLayout::OptionsLayout => {
    //         f.render_widget(Clear, area); //this clears out the background
    //         f.render_widget(options_block, area);
    //     },
    //     _ => {}
    // }
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
