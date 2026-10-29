//! Dev tool: index a folder into a throwaway vault and print the register.
//! `cargo run --example scan --no-default-features -- <folder> [--json]`

use tauri_plugin_aliv_vault::{model::MatterInput, store::Store, Vault};

fn main() {
    let folder = std::env::args().nth(1).expect("usage: scan <folder>");
    let vault = Vault::new(Store::open_in_memory().expect("open"));
    let m = vault.store.create_matter(&MatterInput { name: "scan".into(), ..Default::default() }).expect("matter");
    let json = std::env::args().any(|a| a == "--json");
    let report = vault.ingest_paths(&m.id, &[folder]).expect("ingest");
    if json {
        let grid = vault.store.review_grid(&m.id).expect("grid");
        println!("{}", serde_json::to_string(&serde_json::json!({ "report": report, "grid": grid })).unwrap());
        return;
    }
    println!("{report:?}\n");
    for row in vault.store.review_grid(&m.id).expect("grid") {
        println!("## {}", row.document.file_name);
        for f in row.fields {
            println!("  {:<17} {:.2}  {}", f.key, f.confidence, f.value);
        }
    }
}
