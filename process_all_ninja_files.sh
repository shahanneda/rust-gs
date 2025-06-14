#!/bin/bash

set -e  # Exit on any error

NINJA_DIR="/Users/shahanneda/Documents/projects/rust-gs/splats/ninja"
LOCAL_RS_PATH="/Users/shahanneda/Documents/projects/rust-gs/src/local/local.rs"

# Files to process - excluding files we've already processed or created
FILES=(
    "apple_rotate.ply"
    "bread.ply"
    "orange.ply"
    "watermelon.ply"
    "cake.ply"
    "cake_rotate.ply"
)

# Ensure conda is initialized for this script
eval "$(conda shell.bash hook)"
conda activate cs486

# Process each file
for file in "${FILES[@]}"; do
    echo "===================================================="
    echo "Processing $file"
    echo "===================================================="
    
    # Get base filename without extension
    base_name="${file%.*}"
    
    # Step 1: Create simplified version with increased scale
    # echo "Step 1: Creating simplified version with increased scale"
    # python -u prune.py "$NINJA_DIR/$file" --output_file "$NINJA_DIR/${base_name}_fuller.ply" --method random_scaled --sample_ratio 0.2 --scale_factor 1.5
    
    # Step 2: Create extra full version with even larger scale
    # echo "Step 2: Creating extra full version with larger scale"
    # python -u prune.py "$NINJA_DIR/$file" --output_file "$NINJA_DIR/${base_name}_1.ply" --method random_scaled --sample_ratio 0.1 --scale_factor 3.33
    
    # Step 3: Update local.rs to process the regular simplified version
    echo "Step 3: Creating RKYV file for _fuller version"
    sed -i '' "s|let scene_name = \".*\";|let scene_name = \"ninja/${base_name}\";|" "$LOCAL_RS_PATH"
    
    # Run buildLocal.sh
    echo "Running buildLocal.sh for _fuller version"
    ./buildLocal.sh
    
    # # Step 4: Update local.rs to process the extra full version  
    # echo "Step 4: Creating RKYV file for _extra_full version"
    # sed -i '' "s|let scene_name = \".*\";|let scene_name = \"ninja/${base_name}_extra_full\";|" "$LOCAL_RS_PATH"
    
    # # Run buildLocal.sh
    # echo "Running buildLocal.sh for _extra_full version"
    # ./buildLocal.sh
    
    echo "Completed processing $file"
    echo ""
done

echo "All files processed!" 