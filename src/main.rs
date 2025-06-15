use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PositionsFile {
    result_set: ResultSet,
}

#[derive(Deserialize)]
struct VehiclePosition {
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize)]
struct ResultSet {
    vehicle: Vec<VehiclePosition>,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        return Err(anyhow!("Usage: {} <json_file_path>", args[0]));
    }
    
    let file_path = &args[1];
    
    let contents = fs::read_to_string(file_path)
        .map_err(|err| anyhow!("Error reading file '{}': {}", file_path, err))?;
    
    let positions_file: PositionsFile = serde_json::from_str(&contents)
        .map_err(|err| anyhow!("Error parsing JSON: {}", err))?;
    
    println!("Successfully loaded {} vehicle positions", 
             positions_file.result_set.vehicle.len());
    
    Ok(())
}
