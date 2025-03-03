#!/usr/bin/env python3
import os
import subprocess
import glob

# The bucket name and region
BUCKET = "zimpmodels"
REGION = "us-east-2"

# Find all *_extra_full.ply files
ply_files = glob.glob("splats/ninja/*_extra_full.ply")

# Upload each file to S3
for file_path in ply_files:
    filename = os.path.basename(file_path)
    s3_path = f"s3://{BUCKET}/splats/ninja/{filename}"
    
    print(f"Uploading {file_path} to {s3_path}")
    subprocess.run(["aws", "s3", "cp", file_path, s3_path], check=True)

print(f"Successfully uploaded {len(ply_files)} PLY files to S3.")
print("Files are available at:")
for file_path in ply_files:
    filename = os.path.basename(file_path)
    url = f"https://{BUCKET}.s3.{REGION}.amazonaws.com/splats/ninja/{filename}"
    print(url) 