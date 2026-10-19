slint::include_modules!();

use slint::{SharedString, ModelRc, VecModel, Model, Weak};
use std::rc::Rc;

fn main() -> Result<(), slint::PlatformError> {

    let initial_tabs = vec! [
        TabItem { name: SharedString::from("Main Menu")}
    ];
    let tabs_model = Rc::new(VecModel::from(initial_tabs));

    // Placeholders for app undo/redo stacks
    let mut undo_stack: Vec<Vec<TabItem>> = Vec::new();
    let _redo_stack: Vec<Vec<TabItem>> = Vec::new(); 
    
    let ui = MainWindow::new()?;

    // Tab stuff
    ui.set_tabs(ModelRc::from(tabs_model.clone()));

    let ui_weak = ui.as_weak();
    let tabs_model_clone = tabs_model.clone();
    ui.on_add_tab_clicked(move || {
        let mut current_state = Vec::new();
        for i in 0..(*tabs_model_clone).row_count() {
            if let Some(tab) = (*tabs_model_clone).row_data(i) {
                current_state.push(tab);
            }
        }
        undo_stack.push(current_state);

        let next_index = (*tabs_model_clone).row_count() + 1;
        
        (*tabs_model_clone).push(TabItem {
            name: SharedString::from(format!("Tab {}", next_index)),
        });
    });

    ui.on_undo_clicked(move || {
        println!("Undo requested!");
    });

    ui.on_redo_clicked(move || {
        println!("Redo requested!");
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
