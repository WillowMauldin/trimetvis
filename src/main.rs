use anyhow::{anyhow, Result};
use image::{RgbImage, ImageBuffer, Rgb};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PositionsFile {
    result_set: ResultSet,
}

#[derive(Deserialize, Clone)]
struct VehiclePosition {
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize)]
struct ResultSet {
    vehicle: Vec<VehiclePosition>,
}

fn bucket_vehicles(vehicles: &[VehiclePosition], min_lat: f64, max_lat: f64, min_lon: f64, max_lon: f64) -> [[u32; 256]; 256] {
    let mut matrix = [[0u32; 256]; 256];
    
    let lat_range = max_lat - min_lat;
    let lon_range = max_lon - min_lon;
    
    for vehicle in vehicles {
        let lat_normalized = (vehicle.latitude - min_lat) / lat_range;
        let lon_normalized = (vehicle.longitude - min_lon) / lon_range;
        
        let row = ((lat_normalized * 255.0) as usize).min(255);
        let col = ((lon_normalized * 255.0) as usize).min(255);
        
        matrix[row][col] += 1;
    }
    
    matrix
}

fn export_matrix_as_image(matrix: &[[u32; 256]; 256], max_count: u32, output_path: &str) -> Result<()> {
    let mut img: RgbImage = ImageBuffer::new(256, 256);
    
    for (y, row) in matrix.iter().enumerate() {
        for (x, &count) in row.iter().enumerate() {
            let normalized_value = if max_count > 0 {
                count as f64 / max_count as f64
            } else {
                0.0
            };
            
            let color = colorous::INFERNO.eval_continuous(normalized_value);
            
            img.put_pixel(x as u32, y as u32, Rgb([color.r, color.g, color.b]));
        }
    }
    
    img.save(output_path)?;
    Ok(())
}

fn load_vehicles_by_minute_from_directory(dir_path: &str) -> Result<BTreeMap<u32, Vec<VehiclePosition>>> {
    let mut vehicles_by_minute = BTreeMap::new();
    
    fn visit_dir(dir: &Path, vehicles_map: &mut BTreeMap<u32, Vec<VehiclePosition>>) -> Result<()> {
        let entries = fs::read_dir(dir)
            .map_err(|err| anyhow!("Error reading directory '{}': {}", dir.display(), err))?;
        
        for entry in entries {
            let entry = entry.map_err(|err| anyhow!("Error reading directory entry: {}", err))?;
            let path = entry.path();
            
            if path.is_dir() {
                visit_dir(&path, vehicles_map)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let file_stem = path.file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| anyhow!("Invalid filename: {}", path.display()))?;
                
                let minute = file_stem.parse::<u32>()
                    .map_err(|_| anyhow!("Filename is not a valid minute: {}", file_stem))?;
                
                println!("Processing file: {} (minute {})", path.display(), minute);
                
                let contents = fs::read_to_string(&path)
                    .map_err(|err| anyhow!("Error reading file '{}': {}", path.display(), err))?;
                
                let positions_file: PositionsFile = serde_json::from_str(&contents)
                    .map_err(|err| anyhow!("Error parsing JSON in '{}': {}", path.display(), err))?;
                
                vehicles_map.insert(minute, positions_file.result_set.vehicle);
            }
        }
        
        Ok(())
    }
    
    let dir_path = Path::new(dir_path);
    visit_dir(dir_path, &mut vehicles_by_minute)?;
    
    Ok(vehicles_by_minute)
}

fn get_global_bounds(vehicles_by_minute: &BTreeMap<u32, Vec<VehiclePosition>>) -> Result<(f64, f64, f64, f64)> {
    let mut min_lat = f64::INFINITY;
    let mut max_lat = f64::NEG_INFINITY;
    let mut min_lon = f64::INFINITY;
    let mut max_lon = f64::NEG_INFINITY;
    
    for vehicles in vehicles_by_minute.values() {
        for vehicle in vehicles {
            min_lat = min_lat.min(vehicle.latitude);
            max_lat = max_lat.max(vehicle.latitude);
            min_lon = min_lon.min(vehicle.longitude);
            max_lon = max_lon.max(vehicle.longitude);
        }
    }
    
    if min_lat == f64::INFINITY {
        return Err(anyhow!("No vehicle data found"));
    }
    
    Ok((min_lat, max_lat, min_lon, max_lon))
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        return Err(anyhow!("Usage: {} <n_minutes>", args[0]));
    }
    
    let n_minutes = args[1].parse::<u32>()
        .map_err(|_| anyhow!("n_minutes must be a positive integer"))?;
    
    let data_dir = "data";
    
    if !Path::new(data_dir).exists() {
        return Err(anyhow!("Data directory '{}' does not exist", data_dir));
    }
    
    let vehicles_by_minute = load_vehicles_by_minute_from_directory(data_dir)?;
    
    if vehicles_by_minute.is_empty() {
        return Err(anyhow!("No vehicle data found in the data directory"));
    }
    
    println!("Loaded data for {} minutes", vehicles_by_minute.len());
    
    let (min_lat, max_lat, min_lon, max_lon) = get_global_bounds(&vehicles_by_minute)?;
    println!("Global bounds - Latitude: {:.6} to {:.6}, Longitude: {:.6} to {:.6}", 
             min_lat, max_lat, min_lon, max_lon);
    
    fs::create_dir_all("heatmaps")?;
    
    let minutes: Vec<u32> = vehicles_by_minute.keys().cloned().collect();
    
    for &current_minute in &minutes {
        let start_minute = if current_minute >= n_minutes { 
            current_minute - n_minutes + 1 
        } else { 
            minutes[0] 
        };
        
        let mut combined_vehicles = Vec::new();
        
        for &minute in &minutes {
            if minute >= start_minute && minute <= current_minute {
                combined_vehicles.extend(vehicles_by_minute[&minute].iter().cloned());
            }
        }
        
        if !combined_vehicles.is_empty() {
            let matrix = bucket_vehicles(&combined_vehicles, min_lat, max_lat, min_lon, max_lon);
            
            let mut max_vehicles_in_bucket = 0;
            for row in &matrix {
                for &count in row {
                    max_vehicles_in_bucket = max_vehicles_in_bucket.max(count);
                }
            }
            
            let output_path = format!("heatmaps/heatmap_{:04}.png", current_minute);
            export_matrix_as_image(&matrix, max_vehicles_in_bucket, &output_path)?;
            
            println!("Minute {}: {} vehicles (from minutes {}-{}) -> {}", 
                     current_minute, combined_vehicles.len(), start_minute, current_minute, output_path);
        }
    }
    
    println!("Generated {} heatmaps in the 'heatmaps' directory", minutes.len());
    
    Ok(())
}
