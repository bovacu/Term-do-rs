#[derive(Copy, Clone, PartialEq)]
pub enum FocusedLayout {
    None,
    GroupsLayout,
    TasksLayout
}

#[derive(Copy, Clone, PartialEq)]
pub enum InputMode {
    Navigate,
    Add,
    Edit
}