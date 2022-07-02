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
use tui::style::{Modifier, Style};
use tui::widgets::Clear;

use crate::data_manager::{DataManager, LayoutCommon};
use crate::enums::{FocusedLayout, InputMode};
use crate::group_layout::GroupLayout;
use crate::tasks_layout::{TaskLayout};

use unicode_width::UnicodeWidthStr;

trait LayoutCommonTrait {
    fn handle_input(&mut self, data_manager: &mut DataManager, key_code: crossterm::event::KeyEvent);
     fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>, frame_size: &Rect);
    fn create_and_render_base_block<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>);
     fn create_and_render_item_list<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>, frame_size: &Rect);
    fn create_and_render_edit_mode<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>);

    fn is_in_edit_mode(layout_common: &LayoutCommon) -> bool {
        return LayoutCommon::is_in_edit_mode(layout_common);
    }

    fn render_common_input_mode<B: Backend>(f: &mut Frame<B>, layout_common: &mut LayoutCommon, title: &str, chunk: &Vec<Rect>) {
        if layout_common.is_in_edit_mode() {
            let options_block = Block::default().title(title).borders(Borders::ALL);
            let area = centered_rect(40, 10, chunk[1]);

            layout_common.max_string_width = (area.width as f32 / 1.1) as usize;

            let input = Paragraph::new(layout_common.input[layout_common.starting_rendering_input_point..].as_ref())
                .style( if layout_common.is_in_edit_mode() {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                })
                .block(options_block);

            f.render_widget(Clear, area);
            f.render_widget(input, area);

            f.set_cursor(
                area.x +  layout_common.cursor_pos as u16 + 1 - layout_common.starting_rendering_input_point as u16,
                area.y + 1,
            )
        }
    }

    fn poll_common_keys_input_mode(key_code: &event::KeyEvent, layout_common: &mut LayoutCommon) {
        match key_code.code {
            KeyCode::Char(c) => {
                <TaskLayout as LayoutCommonTrait>::char_input_mode(layout_common, c);
            },
            KeyCode::Backspace => {
                <TaskLayout as LayoutCommonTrait>::backspace_key_input_mode(layout_common);
            },
            KeyCode::Delete => {
                <TaskLayout as LayoutCommonTrait>::delete_key_input_mode(layout_common);
            },
            KeyCode::Left => {
                <TaskLayout as LayoutCommonTrait>::left_key_input_mode(layout_common);
            },
            KeyCode::Right => {
                <TaskLayout as LayoutCommonTrait>::right_key_input_mode(layout_common);
            },
            KeyCode::Esc => {
                <TaskLayout as LayoutCommonTrait>::esc_key_input_mode(layout_common);
            },
            _ => {}
        }
    }

    fn left_key_input_mode(layout_common: &mut LayoutCommon) {
        if layout_common.cursor_pos - layout_common.starting_rendering_input_point > 0 {
            layout_common.cursor_pos -= 1;
        } else {
            if layout_common.starting_rendering_input_point > 0 {
                layout_common.starting_rendering_input_point -= 1;
            }
        }
    }

    fn right_key_input_mode(layout_common: &mut LayoutCommon) {
        if layout_common.cursor_pos - layout_common.starting_rendering_input_point < layout_common.max_string_width &&
            layout_common.cursor_pos < layout_common.input.width() {
            layout_common.cursor_pos += 1;
        } else {
            if layout_common.starting_rendering_input_point < layout_common.input.width() - layout_common.max_string_width &&
                layout_common.cursor_pos < layout_common.input.width() {
                layout_common.starting_rendering_input_point += 1;
            }
        }
    }

    fn esc_key_input_mode(layout_common: &mut LayoutCommon) {
        layout_common.input_mode = InputMode::Navigate;
    }

    fn delete_key_input_mode(layout_common: &mut LayoutCommon) {
        // This is separated as the compiler tells me usize >= 0 is a useless comparison, I know ma' bro
        // but I still need to check if it is equal to 0
        if layout_common.cursor_pos == 0 || layout_common.cursor_pos > 0 {
            if layout_common.cursor_pos == layout_common.input.width() {
                return;
            }
            layout_common.input.remove(layout_common.cursor_pos);
        }
    }

    fn backspace_key_input_mode(layout_common: &mut LayoutCommon) {
        if layout_common.cursor_pos > 0 {
            if layout_common.cursor_pos == layout_common.input.width() {
                layout_common.input.pop();
            } else {
                layout_common.input.remove(layout_common.cursor_pos - 1);
            }
            layout_common.cursor_pos -= 1;
        }

        LayoutCommon::recalculate_input_string_starting_point(layout_common);
    }

    fn char_input_mode(layout_common: &mut LayoutCommon, c: char) {
        if layout_common.cursor_pos == layout_common.input.width() {
            layout_common.input.push(c);
        } else {
            layout_common.input.insert(layout_common.cursor_pos, c);
        }
        layout_common.cursor_pos += 1;

        LayoutCommon::recalculate_input_string_starting_point(layout_common);
    }
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

    pub fn is_in_edit_mode(&self) -> bool {
        <GroupLayout as LayoutCommonTrait>::is_in_edit_mode(&self.group_layout.layout_common) || <TaskLayout as LayoutCommonTrait>::is_in_edit_mode(&self.task_layout.layout_common)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.data_manager.load_state();
    let res =  { run_app(&mut terminal, &mut app) };

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
                if  !app.is_in_edit_mode() {
                    return Ok(());
                }
            } else if key.code == KeyCode::Left && !app.is_in_edit_mode() {
                app.update_state(FocusedLayout::GroupsLayout);
                app.data_manager.selected_task = 0;
            } else if key.code == KeyCode::Right && !app.is_in_edit_mode() {
                app.update_state(FocusedLayout::TasksLayout);
            } else if key.code == KeyCode::Char('u') {
                app.data_manager.load_undo();
                app.data_manager.save_state();
            } else if key.code == KeyCode::Char('r') {
                // app.data_manager.load_redo();
                // app.data_manager.save_state();
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

    <GroupLayout as LayoutCommonTrait>::ui(f, app, &lower_chunks, &f.size());
    <TaskLayout as LayoutCommonTrait>::ui(f, app, &lower_chunks, &f.size());

    <GroupLayout as LayoutCommonTrait>::create_and_render_edit_mode(f, app, &lower_chunks);
    <TaskLayout as LayoutCommonTrait>::create_and_render_edit_mode(f, app, &lower_chunks);
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
