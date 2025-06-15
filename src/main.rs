use anyhow::{anyhow, Result};
use image::{GrayImage, ImageBuffer};
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
    let mut img: GrayImage = ImageBuffer::new(256, 256);
    
    for (y, row) in matrix.iter().enumerate() {
        for (x, &count) in row.iter().enumerate() {
            let intensity = if max_count > 0 {
                ((count as f64 / max_count as f64) * 255.0) as u8
            } else {
                0
            };
            img.put_pixel(x as u32, y as u32, image::Luma([intensity]));
        }
    }
    
    img.save(output_path)?;
    Ok(())
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
    
    let vehicles = &positions_file.result_set.vehicle;
    println!("Successfully loaded {} vehicle positions", vehicles.len());
    
    if !vehicles.is_empty() {
        let mut min_lat = vehicles[0].latitude;
        let mut max_lat = vehicles[0].latitude;
        let mut min_lon = vehicles[0].longitude;
        let mut max_lon = vehicles[0].longitude;
        
        for vehicle in vehicles {
            min_lat = min_lat.min(vehicle.latitude);
            max_lat = max_lat.max(vehicle.latitude);
            min_lon = min_lon.min(vehicle.longitude);
            max_lon = max_lon.max(vehicle.longitude);
        }
        
        println!("Latitude range: {:.6} to {:.6}", min_lat, max_lat);
        println!("Longitude range: {:.6} to {:.6}", min_lon, max_lon);
        
        let matrix = bucket_vehicles(vehicles, min_lat, max_lat, min_lon, max_lon);
        
        let mut total_buckets_used = 0;
        let mut max_vehicles_in_bucket = 0;
        
        for row in &matrix {
            for &count in row {
                if count > 0 {
                    total_buckets_used += 1;
                    max_vehicles_in_bucket = max_vehicles_in_bucket.max(count);
                }
            }
        }
        
        println!("Bucketed into 256x256 matrix:");
        println!("  Buckets with vehicles: {}/65536", total_buckets_used);
        println!("  Max vehicles in single bucket: {}", max_vehicles_in_bucket);
        
        let output_path = "vehicle_heatmap.png";
        export_matrix_as_image(&matrix, max_vehicles_in_bucket, output_path)?;
        println!("Exported heatmap to: {}", output_path);
    }
    
    Ok(())
}
