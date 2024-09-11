use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut config = lalrpop::Configuration::new();
    config.emit_rerun_directives(true);
    config.process_current_dir()
}
