slint::include_modules!();

use slint::{SharedString, ModelRc, VecModel, Model, Weak};
use std::rc::Rc;
use std::cell::{RefCell};
use std::collections::VecDeque;

mod modules;
use modules::{ModuleState, placeholder::PlaceholderState};


pub struct StateRegistry {
    next_id: i32,
    registry: std::collections::HashMap<i32, ModuleState>,
}
impl StateRegistry {
    pub fn new() -> Self {
        let mut initial_regestry = std::collections::HashMap::new();
        initial_regestry.insert(0, ModuleState::MainMenu);

        Self {
            next_id: 1,
            registry: initial_regestry,
        }
    }
    pub fn register_new_module(&mut self, kind: &str) -> i32 {
        let id = self.next_id;
        self.next_id += 1;

        let state = match kind {
            "Main Menu" => ModuleState::MainMenu,
            _ => ModuleState::Placeholder(PlaceholderState::default()),
        };

        self.registry.insert(id, state);
        id
    }
    pub fn get_mut_state(&mut self, id: i32) -> Option<&mut ModuleState> {
        self.registry.get_mut(&id)
    }
}

pub trait UndoableCommand {
    fn name(&self) -> &str;
    fn undo(&mut self, ui: &MainWindow);
    fn redo(&mut self, ui: &MainWindow);
}
struct AddTabCommand {
    module_name: String,
    assigned_id: i32,
    assigned_index: usize,
    tabs_model: Rc<VecModel<TabItem>>,
    active_tab_state: Rc<std::cell::Cell<i32>>,
}
impl UndoableCommand for AddTabCommand {
    fn name(&self) -> &str { "Add Tab" }
    fn undo(&mut self, ui: &MainWindow) {
        if self.tabs_model.row_count() > self.assigned_index {
            self.tabs_model.remove(self.assigned_index);

            let new_focus = (self.assigned_index as i32 - 1).max(0);
            self.active_tab_state.set(new_focus);
            ui.set_active_tab_index(new_focus);
        }
    }
    fn redo(&mut self, ui: &MainWindow) {
        self.tabs_model.insert(self.assigned_index, TabItem {
            id: self.assigned_id,
            kind: SharedString::from(&self.module_name),
        });

        let target = self.assigned_index as i32;
        self.active_tab_state.set(target);
        ui.set_active_tab_index(target);
    
    }
}
struct SwitchTabCommand {
    old_index: i32,
    new_index: i32,
    active_tab_state: Rc<std::cell::Cell<i32>>,
}
impl UndoableCommand for SwitchTabCommand {
    fn name(&self) -> &str { "Switch Tab" }
    fn undo(&mut self, ui: &MainWindow) {
        self.active_tab_state.set(self.old_index);
        ui.set_active_tab_index(self.old_index);
    }
    fn redo(&mut self, ui: &MainWindow) {
        self.active_tab_state.set(self.new_index);
        ui.set_active_tab_index(self.new_index);
    }
}

struct CloseTabCommand {
    removed_index: usize,
    removed_tab: TabItem,
    fallback_focus_index: i32,
    previous_active_index: i32,
    tabs_model: Rc<VecModel<TabItem>>,
    active_tab_state: Rc<std::cell::Cell<i32>>,
}
impl UndoableCommand for CloseTabCommand {
    fn name(&self) -> &str { "Close Tab" }
    fn undo(&mut self, ui: &MainWindow) {
        self.tabs_model.insert(self.removed_index, self.removed_tab.clone());
        
        self.active_tab_state.set(self.previous_active_index);
        ui.set_active_tab_index(self.previous_active_index);
    }
    fn redo(&mut self, ui: &MainWindow) {
        if self.tabs_model.row_count() > self.removed_index {
            self.tabs_model.remove(self.removed_index);
            self.active_tab_state.set(self.fallback_focus_index);
            ui.set_active_tab_index(self.fallback_focus_index);
        }
    }
}

pub struct HistoryManager {
    undo_stack: VecDeque<Box<dyn UndoableCommand>>,
    redo_stack: VecDeque<Box<dyn UndoableCommand>>,
    max_capacity: usize,
}
impl HistoryManager {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(max_capacity),
            redo_stack: VecDeque::with_capacity(max_capacity),
            max_capacity,
        }
    }

    pub fn record_action(&mut self, command: Box<dyn UndoableCommand>) {
        if self.undo_stack.len() >= self.max_capacity { self.undo_stack.pop_front(); }
        self.undo_stack.push_back(command);
        self.redo_stack.clear();
    }

    pub fn execute_undo(&mut self, ui: &MainWindow) {
        if let Some(mut command) = self.undo_stack.pop_back() {
            command.undo(ui);
            self.redo_stack.push_back(command);
        }
    }

    pub fn execute_redo(&mut self, ui: &MainWindow) {
        if let Some(mut command) = self.redo_stack.pop_back() {
            command.redo(ui);
            self.undo_stack.push_back(command);
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {

    // initializing
    let initial_tabs = vec! [
        TabItem { id: 0, kind: SharedString::from("Main Menu")}
    ];
    let tabs_model = Rc::new(VecModel::from(initial_tabs));

    // base infrastructure
    let history_manager = Rc::new(RefCell::new(HistoryManager::new(100)));
    let state_registry = Rc::new(RefCell::new(StateRegistry::new()));
    let processing_history = Rc::new(std::cell::Cell::new(false));
    let active_tab_state = Rc::new(std::cell::Cell::new(0));
    
    let ui = MainWindow::new()?;
    let ui_weak = ui.as_weak();

    //tab change callback
    let active_tab_clone = active_tab_state.clone();
    let switch_history = history_manager.clone();
    let switch_history_lock = processing_history.clone();
    ui.on_tab_changed(move |new_index| {
        let old_index = active_tab_clone.get();

        if old_index == new_index || switch_history_lock.get() {
            active_tab_clone.set(new_index);
            return;
        }

        let cmd = SwitchTabCommand {
            old_index,
            new_index,
            active_tab_state: active_tab_clone.clone(),
        };

        switch_history.borrow_mut().record_action(Box::new(cmd));
        active_tab_clone.set(new_index);
    });
    
    //initial model references
    ui.set_tabs(ModelRc::from(tabs_model.clone()));

    let add_tab_ui_weak = ui_weak.clone();
    let tabs_model_clone = tabs_model.clone();
    let active_tab_state_clone = active_tab_state.clone();
    let history_clone = history_manager.clone();
    let registry_clone = state_registry.clone();
    let add_lock = processing_history.clone();

    ui.on_add_tab(move |module_name| {
        if add_lock.get() { return; }

        let mut registry = registry_clone.borrow_mut();
        let assigned_id = registry.register_new_module(&module_name);
        let current_index = tabs_model_clone.row_count();

        let mut cmd = AddTabCommand {
            module_name: module_name.to_string(),
            assigned_id,
            assigned_index: current_index,
            tabs_model: tabs_model_clone.clone(),
            active_tab_state: active_tab_state_clone.clone(),
        };

        if let Some(gui) = add_tab_ui_weak.upgrade() {
            add_lock.set(true);
            cmd.redo(&gui);
            add_lock.set(false);
        }
        history_clone.borrow_mut().record_action(Box::new(cmd));
    });

    //tab closing callback
    let close_tabs_model = tabs_model.clone();
    let close_history = history_manager.clone();
    let close_active_state = active_tab_state.clone();
    let close_ui_weak = ui_weak.clone();
    let close_lock = processing_history.clone();

    ui.on_close_tab(move |idx| {
        if close_lock.get() { return; }
        let index = idx as usize;
        if close_tabs_model.row_count() <= 1 { return; }

        if let Some(tab_to_remove) = close_tabs_model.row_data(index) {
            let current_active = close_active_state.get();

            let fallback_index = if index > 0 { (index - 1) as i32 } else { 0 };

            let mut cmd = CloseTabCommand {
                removed_index: index,
                removed_tab: tab_to_remove,
                fallback_focus_index: fallback_index,
                previous_active_index: current_active,
                tabs_model: close_tabs_model.clone(),
                active_tab_state: close_active_state.clone(),
            };

            if let Some(gui) = close_ui_weak.upgrade() {
                close_lock.set(true);
                cmd.redo(&gui);
                close_lock.set(false);
            }
            close_history.borrow_mut().record_action(Box::new(cmd));
        }
    });

    let ui_weak_clone1 = ui_weak.clone();
    let ui_weak_clone2 = ui_weak.clone();
    let history_manager_clone1 = history_manager.clone();
    let history_manager_clone2 = history_manager.clone();
    let undo_lock = processing_history.clone();
    let redo_lock = processing_history.clone();
    ui.on_undo_clicked(move || {
        if let Some(gui) = ui_weak_clone1.upgrade() {
            undo_lock.set(true);
            history_manager_clone1.borrow_mut().execute_undo(&gui);
            undo_lock.set(false);
        }
    });

    ui.on_redo_clicked(move || {
        if let Some(gui) = ui_weak_clone2.upgrade() {
            redo_lock.set(true);
            history_manager_clone2.borrow_mut().execute_redo(&gui);
            redo_lock.set(false);
        }
    });

    let test_registry = state_registry.clone();
    let test_history = history_manager.clone();
    let test_ui_weak = ui_weak.clone();

    ui.on_increment_placeholder_counter(move |tab_id| {
        let mut registry = test_registry.borrow_mut();
        if let Some(ModuleState::Placeholder(state)) = registry.get_mut_state(tab_id) {
            state.click_count += 1;
            
            if let Some(gui) = test_ui_weak.upgrade() {
                gui.set_placeholder_display_count(state.click_count);
            }
        }
    });

    // tab scrollbar
    tab_bar_scroll(ui_weak, &ui);

    ui.run()
}


fn tab_bar_scroll(ui_weak: Weak<MainWindow>, ui: &MainWindow) {
    let scroll_state = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
    let scroll_state_clone = scroll_state.clone();

    ui.on_hold_scroll_changed(move |direction| {
        scroll_state_clone.store(direction, std::sync::atomic::Ordering::SeqCst);
        
        if direction != 0 {

            let thread_weak = ui_weak.clone(); 
            let current_direction = scroll_state_clone.clone();

            std::thread::spawn(move || {
                loop {
                    let dir = current_direction.load(std::sync::atomic::Ordering::SeqCst);
                    if dir == 0 { break; }
    
                    let loop_weak = thread_weak.clone();
                    
                    let _ = loop_weak.upgrade_in_event_loop(move |gui| {
                        let current_x = gui.get_tab_scroll_x(); 

                        if dir == -1 {
                            let next_x = current_x + 50.0;
                            gui.set_tab_scroll_x(if next_x > 0.0 { 0.0 } else { next_x });
                        } else if dir == 1 {
                            let next_x = current_x - 50.0;
                            gui.set_tab_scroll_x(next_x);
                        }
                    });

                    std::thread::sleep(std::time::Duration::from_millis(16));
                }
            });
        }
    });
}
