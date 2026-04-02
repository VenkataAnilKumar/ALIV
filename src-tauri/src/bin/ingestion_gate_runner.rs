use std::env;

use aliv_desktop_lib::{
    core::ingestion::IngestionService,
    infra::{db::Database, python_bridge::PythonBridge},
    models::contracts::IngestionRequest,
};

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let workspace_path = args
        .iter()
        .position(|value| value == "--workspace")
        .and_then(|index| args.get(index + 1))
        .cloned()
        .unwrap_or_else(|| ".".to_string());
    let run_incremental = args.iter().any(|value| value == "--incremental");
    let run_twice = args.iter().any(|value| value == "--twice");

    let database = Database::connect_default().map_err(|err| err.to_string())?;
    let python_bridge = PythonBridge::new().map_err(|err| err.to_string())?;

    database
        .register_workspace_folder(&workspace_path)
        .map_err(|err| err.to_string())?;

    execute_ingestion(&database, &python_bridge, &workspace_path, false)?;

    if run_incremental || run_twice {
        execute_ingestion(&database, &python_bridge, &workspace_path, true)?;
    }

    println!("ingestion runs completed");
    Ok(())
}

fn execute_ingestion(
    database: &Database,
    python_bridge: &PythonBridge,
    workspace_path: &str,
    incremental: bool,
) -> Result<(), String> {
    let service = IngestionService;
    let request = IngestionRequest {
        workspace_path: workspace_path.to_string(),
        incremental,
    };

    let run = service
        .run(database, python_bridge, &request)
        .map_err(|err| err.to_string())?;

    println!("{}", service.summary_detail(&run));

    Ok(())
}
