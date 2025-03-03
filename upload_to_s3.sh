#!/bin/bash

# Exit on any error
set -e

# Make the script executable
chmod +x upload_to_s3.sh

# Check if AWS credentials are configured
if ! aws sts get-caller-identity &>/dev/null; then
  echo "AWS credentials not configured. Please fill in aws_credentials.txt and run:"
  echo "cat aws_credentials.txt > ~/.aws/credentials"
  exit 1
fi

# The bucket name and region
BUCKET="zimpmodels"
REGION="us-east-2"

# Make sure we're in the right directory
cd "$(dirname "$0")"

# Upload all *_extra_full.rkyv files
echo "Uploading *_extra_full.rkyv files to S3..."
for file in splats/ninja/*_extra_full.rkyv; do
  filename=$(basename "$file")
  echo "Uploading $file to s3://$BUCKET/splats/ninja/$filename"
  aws s3 cp "$file" "s3://$BUCKET/splats/ninja/$filename"
done

echo "All files uploaded successfully."
echo "Now updating index.html to include new models in the dropdown..."

# Backup the original file
cp index.html index.html.bak

# Create a temporary file for the MODEL_LIST entries
temp_file=$(mktemp)

# Start with the existing entries
grep -A20 "const MODEL_LIST" index.html | grep -v "Extra Full" | grep -v "];" > "$temp_file"

# Add the new entries
for file in splats/ninja/*_extra_full.rkyv; do
  filename=$(basename "$file")
  # Get the base name without _extra_full.rkyv
  basename=${filename%_extra_full.rkyv}
  # Create a more readable display name with spaces and proper capitalization
  displayname=$(echo "$basename" | sed 's/_/ /g' | sed 's/\b\(.\)/\u\1/g')
  
  # Create the model entry with proper S3 URL
  S3_URL="https://$BUCKET.s3.$REGION.amazonaws.com/splats/ninja/$filename"
  echo "			{ display: \"${displayname} Extra Full\", file: \"$S3_URL\" }," >> "$temp_file"
done

# Add the closing bracket
echo "		];" >> "$temp_file"

# Replace the MODEL_LIST in the original file
sed -i.bak3 "/const MODEL_LIST/,/];/c\\		const MODEL_LIST = [" index.html
cat "$temp_file" >> index.html
sed -i.bak4 -e '/^		const MODEL_LIST = \[$/,/];/d' -e '/const MODEL_LIST = \[/r'"$temp_file" index.html

# Clean up temporary file
rm "$temp_file"

echo "index.html updated. The original file is backed up as index.html.bak."
echo "Done!" 