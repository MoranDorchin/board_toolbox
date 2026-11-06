use crate::UndoableCommand;
use crate::MainWindow;
use crate::modules::ModuleState;
use std::rc::Rc;
use std::cell::RefCell;
use crate::StateRegistry;

#[derive(Debug, Clone)]
pub struct PlaceholderState {
    pub click_count: i32,
}

//undoable/redoable action
pub struct IncrementCounterCommand {
    tab_id: i32,
    registry: Rc<RefCell<StateRegistry>>,
}

impl IncrementCounterCommand {
    pub fn new(tab_id: i32, registry: Rc<RefCell<StateRegistry>>) -> Self {
        Self { tab_id, registry }
    }

    fn modify_count(&mut self, ui: &MainWindow, delta: i32) {
        let mut reg = self.registry.borrow_mut();
        if let Some(ModuleState::Placeholder(state)) = reg.get_mut_state(self.tab_id) {
            state.click_count += delta;
        }
    }
}

impl UndoableCommand for IncrementCounterCommand {
    fn name(&self) -> &str { "Increment Counter"}

    fn undo(&mut self, ui: &MainWindow) {
        self.modify_count(ui, -1);
    }
    
    fn redo(&mut self, ui: &MainWindow) {
        self.modify_count(ui, 1);
    }
}

impl Default for PlaceholderState {
    fn default() -> Self {
        Self { click_count: 0 }
    }
}