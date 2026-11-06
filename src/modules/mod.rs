pub mod placeholder;

use placeholder::PlaceholderState;

#[derive(Debug, Clone)]
pub enum ModuleState {
    MainMenu,
    Placeholder(PlaceholderState),
}