use std::sync::{Arc, Mutex};

use aspire::{Control, Warning};

#[test]
fn custom_logger() {
    let messages: Arc<Mutex<Vec<(Warning, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let msgs = messages.clone();

    let mut ctl = Control::with_logger(
        &[],
        Some(move |warning: Warning, msg: &str| {
            msgs.lock().unwrap().push((warning, msg.to_string()));
        }),
        20,
    )
    .unwrap();

    // An undefined atom warning
    ctl.add("base", &[], "a :- b.").unwrap();
    ctl.ground_base().unwrap();

    let msgs = messages.lock().unwrap();
    assert!(!msgs.is_empty());
    assert!(msgs.iter().any(|(w, _)| *w == Warning::AtomUndefined));
}

#[test]
fn no_logger_still_works() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "a.").unwrap();
    ctl.ground_base().unwrap();
}
