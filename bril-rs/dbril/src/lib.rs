use salsa::Event;

#[salsa::db]
#[derive(Default)]
pub struct BrilDatabaseImpl {
    storage: salsa::Storage<Self>,
    // The logs are only used for testing and demonstrating reuse:
    // logs: Arc<Mutex<Option<Vec<String>>>>,
}

#[salsa::db]
impl salsa::Database for BrilDatabaseImpl {
    fn salsa_event(&self, event: &dyn Fn() -> Event) {
        let event = event();
        eprintln!("Event: {event:?}");
        // Log interesting events, if logging is enabled
        /*  if let Some(logs) = &mut *self.logs.lock().unwrap() {
            // only log interesting events
            if let salsa::EventKind::WillExecute { .. } = event.kind {
                logs.push(format!("Event: {event:?}"));
            }
        } */
    }
}

#[salsa::input]
pub struct ProgramSource {
    #[return_ref]
    pub text: String,
}
