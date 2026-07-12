use ofm_core::state::StateManager;

fn main() {
    println!("OpenFootManager Handheld initialized");

    let state = StateManager::new();

    // Prove the shared application layer is reachable without Tauri.
    let result = ofm_app::live_match::get_match_snapshot(&state);
    match result {
        Err(e) => println!("ofm_app call succeeded (expected error): {e}"),
        Ok(_) => println!("ofm_app call returned unexpected Ok"),
    }

    println!("Done");
}
