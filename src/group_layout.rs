use crossterm::event::KeyCode;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::{Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, BorderType, Clear, ListItem, Paragraph};
use crate::{App, centered_rect, DataManager, FocusedLayout, LayoutState};

use tui::{widgets::{List}};

use unicode_width::UnicodeWidthStr;
use crate::data_manager::GroupItem;
use crate::enums::InputMode;

pub struct GroupLayout {
    input_mode: InputMode,
    input: String,
    cursor_pos: usize
}

impl GroupLayout {
    pub fn new() -> GroupLayout {
        GroupLayout {
            input_mode: InputMode::Navigate,
            input: String::new(),
            cursor_pos: 0
        }
    }
}

impl LayoutState for GroupLayout {

    fn is_in_edit_mode(&self) -> bool {
        return self.input_mode == InputMode::Add || self.input_mode == InputMode::Edit;
    }

    fn handle_input(&mut self, data_manager: &mut DataManager, key_code: crossterm::event::KeyEvent) {
        match self.input_mode {
            InputMode::Navigate => {
                match key_code.code {
                    KeyCode::Char('a') => {
                        self.input_mode = InputMode::Add;
                        self.input = String::new();
                        self.cursor_pos = self.input.width();
                    },
                    KeyCode::Esc => {
                        if self.input_mode != InputMode::Navigate { return; }
                        self.input_mode = InputMode::Navigate;
                    },
                    KeyCode::Char('e') => {
                        self.input_mode = InputMode::Edit;
                        self.input = data_manager.get_group(data_manager.selected_group).name.clone();
                        self.cursor_pos = self.input.width();
                    },
                    KeyCode::Char('d') => {
                        data_manager.delete_group_item(data_manager.selected_group);
                        data_manager.save_state();
                        data_manager.selected_group = 0;
                    },
                    KeyCode::Up => {
                        if data_manager.selected_group > 0 {
                            data_manager.selected_group -= 1;
                        }
                    },
                    KeyCode::Down => {
                        if data_manager.selected_group < data_manager.get_group_items().len() - 1 {
                            data_manager.selected_group += 1;
                        }
                    }
                    _ => {}
                }
            },
            InputMode::Add => {
                match key_code.code {
                    KeyCode::Enter => {
                        let mut gi = GroupItem::new(data_manager);
                        gi.name = self.input.drain(..).collect();
                        data_manager.add_group_item(gi);
                        self.input_mode = InputMode::Navigate;
                        data_manager.save_state();
                    },
                    KeyCode::Char(c) => {
                        if self.cursor_pos == self.input.width() {
                            self.input.push(c);
                        } else {
                            self.input.insert(self.cursor_pos, c);
                        }
                        self.cursor_pos += 1;
                    },
                    KeyCode::Backspace => {
                        if self.cursor_pos > 0 {
                            if self.cursor_pos == self.input.width() {
                                self.input.pop();
                            } else {
                                self.input.remove(self.cursor_pos - 1);
                            }
                            self.cursor_pos -= 1;
                        }
                    },
                    KeyCode::Delete => {
                        if self.cursor_pos > 0 {
                            if self.cursor_pos == self.input.width() {
                                return;
                            }
                            self.input.remove(self.cursor_pos);
                        }
                    },
                    KeyCode::Left => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                        }
                    },
                    KeyCode::Right => {
                        if self.cursor_pos < self.input.width() {
                            self.cursor_pos += 1;
                        }
                    },
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Navigate;
                    },
                    _ => {}
                }
            },
            InputMode::Edit => {
                match key_code.code {
                    KeyCode::Enter => {
                        data_manager.edit_group_item(data_manager.selected_group, self.input.drain(..).collect());
                        self.input_mode = InputMode::Navigate;
                        data_manager.save_state();
                    },
                    KeyCode::Char(c) => {
                        if self.cursor_pos == self.input.width() {
                            self.input.push(c);
                        } else {
                            self.input.insert(self.cursor_pos, c);
                        }
                        self.cursor_pos += 1;
                    },
                    KeyCode::Backspace => {
                        if self.cursor_pos > 0 {
                            if self.cursor_pos == self.input.width() {
                                self.input.pop();
                            } else {
                                self.input.remove(self.cursor_pos - 1);
                            }
                            self.cursor_pos -= 1;
                        }
                    },
                    KeyCode::Delete => {
                        if self.cursor_pos > 0 {
                            if self.cursor_pos == self.input.width() {
                                return;
                            }
                            self.input.remove(self.cursor_pos);
                        }
                    },
                    KeyCode::Left => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                        }
                    },
                    KeyCode::Right => {
                        if self.cursor_pos < self.input.width() {
                            self.cursor_pos += 1;
                        }
                    },
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Navigate;
                    },
                    _ => {}
                }
            }
        }
    }

    fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>, frame_size: &Rect) {
        GroupLayout::create_and_render_base_block(f, app, chunk);
        GroupLayout::create_and_render_item_list(f, app, chunk, frame_size);
    }

    fn create_and_render_base_block<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>) {
        let mut groups_block = Block::default()
            .title("Groups")
            .borders(Borders::ALL)
            .style(Style::default());

        if app.focused_layout == FocusedLayout::GroupsLayout {
            groups_block = groups_block.style(Style::default().add_modifier(Modifier::BOLD))
                .border_type(BorderType::Thick);
        }

        f.render_widget(groups_block, chunk[0]);
    }

    fn create_and_render_item_list<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>, frame_size: &Rect) {
        let area = centered_rect(85, 85, chunk[0]);
        let mut items_list: Vec<ListItem> = Vec::new();

        let height = ListItem::new("Hello").style(Style::default()).height();
        let max_lines : usize = (frame_size.height as usize / (2 * height)) as usize;
        let selected_group = app.data_manager.selected_group;
        let showing_start_item = if selected_group > max_lines { selected_group - max_lines } else { 0 };

        for i in showing_start_item..app.data_manager.get_group_items().len() {
            let mut line = String::new();
            let group_name = app.data_manager.get_group_items()[i].name.as_str();
            line.push_str(group_name);

            if i == app.data_manager.selected_group {
                line = "  ".to_string();
                line.push_str(group_name);
                items_list.push(ListItem::new(line).style(Style::default().fg(Color::Cyan).remove_modifier(Modifier::BOLD)));
                continue;
            }

            line = "  ".to_string();
            line.push_str(group_name);
            items_list.push(ListItem::new(line).style(Style::default().remove_modifier(Modifier::BOLD)));
        }

        let items = List::new(items_list)
            .block(Block::default().borders(Borders::NONE))
            .highlight_style(
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(items, area);
    }

    fn create_and_render_edit_mode<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>) {
        if app.group_layout.is_in_edit_mode() {
            let title : String = if app.group_layout.input_mode == InputMode::Add { "Add group".to_string() } else { "Edit group".to_string() };
            let options_block = Block::default().title(title).borders(Borders::ALL);
            let area = centered_rect(40, 10, chunk[1]);

            let input = Paragraph::new(app.group_layout.input.as_ref())
                .style( if app.group_layout.is_in_edit_mode() {
                            Style::default().add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        })
                .block(options_block);

            f.render_widget(Clear, area);
            f.render_widget(input, area);


            if app.group_layout.is_in_edit_mode() {
                f.set_cursor(
                    // Put cursor past the end of the input text
                    area.x +  app.group_layout.cursor_pos as u16 + 1,
                    // Move one line down, from the border to the input line
                    area.y + 1,
                )
            }
        }
    }
}

