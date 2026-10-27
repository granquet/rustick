slint::include_modules!();
use crate::sequencer::Event;
use std::sync::mpsc;
use std::thread;

pub fn run(rx: mpsc::Receiver<Event>, nnotes: i32) {
    let window = MainWindow::new().expect("unable to create main window");
    let weak = window.as_weak();

    window.set_divisions(nnotes);

    thread::spawn(move || {
        for event in rx {
            let _ = weak.upgrade_in_event_loop(move |ui| match event {
                Event::Click(_) => ui.set_active(ui.get_active() + 1),
                Event::Bar(_) => ui.set_active(-1),
            });
        }
    });

    let _ = window.run();
}
