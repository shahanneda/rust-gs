#!/usr/bin/env python3
import numpy as np
import os
from plyfile import PlyData, PlyElement
from sklearn.cluster import DBSCAN
import argparse
from tqdm import tqdm

def read_ply(file_path):
    """Read PLY file and return vertex data as numpy array."""
    print(f"Reading PLY file: {file_path}")
    ply_data = PlyData.read(file_path)
    vertex_data = ply_data['vertex']
    
    print(f"Vertex data type: {type(vertex_data)}")
    
    # Get header information
    header_text = ""
    with open(file_path, 'rb') as f:
        line = f.readline().decode('ascii').strip()
        header_text += line + "\n"
        while line != "end_header":
            line = f.readline().decode('ascii').strip()
            header_text += line + "\n"
    
    # Convert PlyElement to numpy array
    vertex_array = vertex_data.data
    print(f"Vertex array type: {type(vertex_array)}")
    print(f"Vertex array shape: {vertex_array.shape}")
    print(f"Vertex array dtype: {vertex_array.dtype}")
    
    return vertex_array, header_text

def random_sampling(vertex_data, sampling_ratio):
    """Randomly sample points from the point cloud."""
    print(f"Randomly sampling {sampling_ratio*100:.2f}% of points...")
    n_points = len(vertex_data)
    n_samples = int(n_points * sampling_ratio)
    
    # Generate random indices
    indices = np.random.choice(n_points, size=n_samples, replace=False)
    
    # Create new vertex data with only the sampled points
    sampled_data = vertex_data[indices]
    
    print(f"Randomly sampled from {n_points} to {n_samples} points")
    return sampled_data

def random_sampling_with_scale(vertex_data, sampling_ratio, scale_factor=1.5):
    """Randomly sample points and increase their scale to make them appear larger."""
    print(f"Randomly sampling {sampling_ratio*100:.2f}% of points with scale factor {scale_factor}...")
    n_points = len(vertex_data)
    n_samples = int(n_points * sampling_ratio)
    
    # Generate random indices
    indices = np.random.choice(n_points, size=n_samples, replace=False)
    
    # Create new vertex data with only the sampled points
    sampled_data = vertex_data[indices].copy()
    
    # Increase scale of the points
    print(f"Adjusting scale parameters by factor of {scale_factor}")
    if 'scale_0' in vertex_data.dtype.names:
        sampled_data['scale_0'] *= scale_factor
    if 'scale_1' in vertex_data.dtype.names:
        sampled_data['scale_1'] *= scale_factor
    if 'scale_2' in vertex_data.dtype.names:
        sampled_data['scale_2'] *= scale_factor
    
    print(f"Randomly sampled from {n_points} to {n_samples} points with increased scale")
    return sampled_data

def voxel_grid_sampling(vertex_data, voxel_size):
    """Sample points using voxel grid approach."""
    print(f"Performing voxel grid sampling with voxel size: {voxel_size}...")
    
    # Extract positions
    positions = np.vstack([
        vertex_data['x'],
        vertex_data['y'],
        vertex_data['z']
    ]).T
    
    # Calculate voxel indices for each point
    voxel_indices = np.floor(positions / voxel_size).astype(int)
    
    # Create a dictionary to store points in each voxel
    voxel_dict = {}
    for i, idx in enumerate(voxel_indices):
        voxel_key = tuple(idx)
        if voxel_key not in voxel_dict:
            voxel_dict[voxel_key] = []
        voxel_dict[voxel_key].append(i)
    
    # Get property names from the vertex data
    property_names = vertex_data.dtype.names
    print(f"Property names: {property_names}")
    
    # Create dictionary to store the sampled vertex data
    sampled_data = {name: [] for name in property_names}
    
    # For each voxel, compute the centroid of the contained points
    print(f"Combining points from {len(voxel_dict)} voxels...")
    for voxel_indices in tqdm(voxel_dict.values()):
        if len(voxel_indices) == 1:
            # If there's only one point in the voxel, just use it
            idx = voxel_indices[0]
            for name in property_names:
                sampled_data[name].append(vertex_data[name][idx])
        else:
            # Otherwise, compute average for each property
            for name in property_names:
                # Special handling for rotation quaternions - we take a representative
                if name.startswith('rot_'):
                    # Just use the first point's rotation as a representative
                    sampled_data[name].append(vertex_data[name][voxel_indices[0]])
                else:
                    # For other properties, calculate the mean
                    values = [vertex_data[name][i] for i in voxel_indices]
                    mean_value = np.mean(values)
                    sampled_data[name].append(mean_value)
    
    # Convert to structured array
    dtype = [(name, vertex_data[name].dtype) for name in property_names]
    sampled_vertex = np.empty(len(sampled_data[property_names[0]]), dtype=dtype)
    
    for name in property_names:
        sampled_vertex[name] = sampled_data[name]
    
    print(f"Reduced from {len(vertex_data)} to {len(sampled_vertex)} points using voxel grid sampling")
    return sampled_vertex

def cluster_points(vertex_data, eps=0.01, min_samples=5):
    """Cluster points based on spatial proximity using DBSCAN."""
    print("Extracting positions for clustering...")
    # Extract positions for clustering
    positions = np.vstack([
        vertex_data['x'],
        vertex_data['y'],
        vertex_data['z']
    ]).T
    
    print(f"Clustering {len(positions)} points with eps={eps}, min_samples={min_samples}...")
    # Perform clustering
    db = DBSCAN(eps=eps, min_samples=min_samples, n_jobs=-1)
    clusters = db.fit_predict(positions)
    
    # Count number of clusters
    n_clusters = len(set(clusters)) - (1 if -1 in clusters else 0)
    n_noise = list(clusters).count(-1)
    print(f"Found {n_clusters} clusters and {n_noise} noise points")
    
    return clusters

def simplify_point_cloud(vertex_data, clusters):
    """Simplify point cloud by averaging points within each cluster."""
    print("Simplifying point cloud...")
    
    # Property names in the vertex_data
    property_names = vertex_data.dtype.names
    
    # Create dictionary to store the simplified vertex data
    simplified_data = {name: [] for name in property_names}
    
    # Handle noise points (labeled as -1) separately
    noise_indices = np.where(clusters == -1)[0]
    for idx in noise_indices:
        for name in property_names:
            simplified_data[name].append(vertex_data[name][idx])
    
    # Process each cluster
    unique_clusters = set(clusters) - {-1}  # Exclude noise points
    for cluster_id in tqdm(unique_clusters):
        # Get indices of points in this cluster
        cluster_indices = np.where(clusters == cluster_id)[0]
        
        # If there's only one point in the cluster, just add it
        if len(cluster_indices) == 1:
            idx = cluster_indices[0]
            for name in property_names:
                simplified_data[name].append(vertex_data[name][idx])
            continue
        
        # Otherwise, average the points
        for name in property_names:
            # Special handling for rotation quaternions - we take a representative instead of averaging
            if name.startswith('rot_'):
                # Just use the first point's rotation as a representative
                simplified_data[name].append(vertex_data[name][cluster_indices[0]])
            else:
                # For other properties, calculate the mean
                mean_value = np.mean([vertex_data[name][i] for i in cluster_indices])
                simplified_data[name].append(mean_value)
    
    # Convert to structured array
    dtype = [(name, vertex_data[name].dtype) for name in property_names]
    simplified_vertex = np.empty(len(simplified_data[property_names[0]]), dtype=dtype)
    
    for name in property_names:
        simplified_vertex[name] = simplified_data[name]
    
    print(f"Reduced from {len(vertex_data)} to {len(simplified_vertex)} points")
    return simplified_vertex

def save_ply(vertex_data, output_path, header_text):
    """Save vertex data to a PLY file."""
    print(f"Saving simplified PLY to: {output_path}")
    
    # Create PlyElement
    vertex_element = PlyElement.describe(vertex_data, 'vertex')
    
    # Create PlyData and write to file
    ply_data = PlyData([vertex_element], text=False)
    ply_data.write(output_path)
    print(f"Saved file with {len(vertex_data)} points")

def main():
    parser = argparse.ArgumentParser(description='Simplify PLY point cloud by voxelization or clustering.')
    parser.add_argument('input_file', help='Input PLY file path')
    parser.add_argument('--output_file', help='Output PLY file path (default: input_file_simplified.ply)')
    parser.add_argument('--method', choices=['voxel', 'dbscan', 'random', 'random_scaled'], default='random_scaled',
                        help='Method for simplification: voxel (voxel grid), dbscan (clustering), random sampling, or random_scaled (random sampling with scale adjustment)')
    parser.add_argument('--voxel_size', type=float, default=0.05, 
                        help='Size of voxels for voxel grid sampling (larger value = more aggressive reduction)')
    parser.add_argument('--eps', type=float, default=0.1, 
                        help='DBSCAN eps parameter (clustering radius)')
    parser.add_argument('--min_samples', type=int, default=10, 
                        help='DBSCAN min_samples parameter')
    parser.add_argument('--sample_ratio', type=float, default=0.2, 
                        help='Random sampling ratio (0.0-1.0) for initial downsampling')
    parser.add_argument('--scale_factor', type=float, default=1.5,
                        help='Factor to increase point scales (only for random_scaled method)')
    args = parser.parse_args()
    
    # Set default output file if not provided
    if not args.output_file:
        base_name = os.path.splitext(args.input_file)[0]
        args.output_file = f"{base_name}_simplified.ply"
    
    # Read input PLY file
    vertex_data, header_text = read_ply(args.input_file)
    print(f"Original point cloud has {len(vertex_data)} points")
    
    # Apply the selected simplification method
    if args.method == 'random':
        # Simple random sampling
        simplified_data = random_sampling(vertex_data, args.sample_ratio)
    
    elif args.method == 'random_scaled':
        # Random sampling with scale adjustment
        simplified_data = random_sampling_with_scale(vertex_data, args.sample_ratio, args.scale_factor)
    
    elif args.method == 'voxel':
        # Voxel grid sampling (most efficient for very large point clouds)
        simplified_data = voxel_grid_sampling(vertex_data, args.voxel_size)
    
    elif args.method == 'dbscan':
        # DBSCAN clustering (better quality but slower)
        # First do random sampling if the point cloud is very large
        if len(vertex_data) > 1000000:
            print(f"Point cloud is very large ({len(vertex_data)} points). Applying random sampling first.")
            vertex_data = random_sampling(vertex_data, args.sample_ratio)
        
        # Now do clustering
        clusters = cluster_points(vertex_data, eps=args.eps, min_samples=args.min_samples)
        simplified_data = simplify_point_cloud(vertex_data, clusters)
    
    # Save simplified point cloud
    save_ply(simplified_data, args.output_file, header_text)
    
    print(f"Point cloud simplified from {len(vertex_data)} to {len(simplified_data)} points")
    print(f"Reduction: {(1 - len(simplified_data)/len(vertex_data))*100:.2f}%")

if __name__ == "__main__":
    main()
