# TriMetVis

A visualization tool for TriMet vehicle position data that generates temporal heatmaps and videos.

## What it does

TriMetVis processes JSON files containing vehicle position data and creates:
- Individual heatmap images showing vehicle density over geographic areas
- Time-lapse videos showing how vehicle patterns change over time
- Uses the INFERNO colormap for visually appealing and perceptually uniform heat visualization

## Prerequisites

- Rust (latest stable version)
- ffmpeg (for video generation)

## Usage

### 1. Data Collection

For continuous data collection, use the included systemd service:

1. **Configure the service file:**
Edit `trimet-collection.service` to set your TriMet App ID and correct paths:
```ini
Environment=TRIMET_APP_ID=your_app_id_here
ExecStart=/path/to/your/trimetvis/collect.sh
WorkingDirectory=/path/to/your/trimetvis
```

2. **Install and enable the service and timer:**
```bash
# Copy service and timer files to systemd directory
sudo cp trimet-collection.service /etc/systemd/system/
sudo cp trimet-collection.timer /etc/systemd/system/

# Reload systemd and enable the timer (which will run the service every minute)
sudo systemctl daemon-reload
sudo systemctl enable trimet-collection.timer
sudo systemctl start trimet-collection.timer
```

The timer runs data collection every minute automatically.

### 2. Generate heatmaps

```bash
# Generate heatmaps with a 5-minute rolling window
cargo run -- 5

# Use different window sizes
cargo run -- 10  # 10-minute window
cargo run -- 1   # 1-minute window
```

This creates PNG heatmap images in the `heatmaps/` directory.

### 3. Create video

```bash
# Make the script executable (first time only)
chmod +x create_heatmap_video.sh

# Create video with default settings (2 fps)
./create_heatmap_video.sh

# Create video with custom name and framerate
./create_heatmap_video.sh my_video.mp4 3
```

## Output

- **Heatmaps**: Individual PNG files in `heatmaps/` directory
- **Video**: MP4 file showing temporal evolution of vehicle patterns
- **Colors**: Uses INFERNO colormap (black → red → orange → yellow → white) for heat intensity

