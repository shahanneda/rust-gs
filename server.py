#!/usr/bin/env python3
import http.server
import socketserver
import os
import sys
import mimetypes
from urllib.parse import urlparse

PORT = 5502

# Ensure proper MIME types are registered
mimetypes.add_type('application/javascript', '.js')
mimetypes.add_type('application/wasm', '.wasm')
mimetypes.add_type('application/octet-stream', '.rkyv')
mimetypes.add_type('application/octet-stream', '.ply')

class CORSHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    # Add proper MIME types for JavaScript modules
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        '.js': 'application/javascript',
        '.mjs': 'application/javascript',
        '.wasm': 'application/wasm',
        '.rkyv': 'application/octet-stream',
        '.ply': 'application/octet-stream',
        '.json': 'application/json',
    }
    
    def end_headers(self):
        # Add headers needed for SharedArrayBuffer
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        
        # Add CORS headers
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        
        super().end_headers()
    
    def do_OPTIONS(self):
        # Handle preflight requests
        self.send_response(200)
        self.end_headers()
    
    def serve_file(self, file_path, content_type=None):
        """Helper method to serve a file with proper headers"""
        if not os.path.exists(file_path) or not os.path.isfile(file_path):
            return False
            
        self.send_response(200)
        
        # Set content type based on file extension or provided content_type
        if content_type:
            self.send_header("Content-Type", content_type)
        else:
            ext = os.path.splitext(file_path)[1]
            if ext in self.extensions_map:
                self.send_header("Content-Type", self.extensions_map[ext])
        
        # Add content-length header
        fs = os.fstat(os.open(file_path, os.O_RDONLY))
        self.send_header("Content-Length", str(fs[6]))
        
        # Add Cross-Origin-Resource-Policy header for all resources
        self.send_header("Cross-Origin-Resource-Policy", "cross-origin")
        
        self.end_headers()
        
        # Send the file content
        with open(file_path, 'rb') as f:
            try:
                self.copyfile(f, self.wfile)
            except BrokenPipeError:
                # Handle broken pipe errors gracefully
                print("Broken pipe error - client likely disconnected", file=sys.stderr)
        
        return True
    
    def serve_js_module(self, content):
        """Helper method to serve a JavaScript module string"""
        self.send_response(200)
        self.send_header("Content-Type", "application/javascript")
        self.send_header("Cross-Origin-Resource-Policy", "cross-origin")
        self.end_headers()
        self.wfile.write(content.encode('utf-8'))
        return True
    
    def do_GET(self):
        # Parse the URL path
        parsed_path = urlparse(self.path)
        path = parsed_path.path
        
        # Handle root path
        if path == "/":
            path = "/index.html"
        
        # Construct the file path
        file_path = os.path.join(os.getcwd(), path[1:])
        
        # First try to serve the file directly if it exists
        if self.serve_file(file_path):
            return
        
        # Special handling for pkg directory requests
        if path.startswith("/pkg/"):
            # Check if this is a worker import request for workerHelpers.js
            if "workerHelpers.js" in path:
                worker_helpers_path = os.path.join(os.getcwd(), "pkg/snippets/wasm-bindgen-rayon-38edf6e439f6d70d/src/workerHelpers.js")
                if self.serve_file(worker_helpers_path, "application/javascript"):
                    return
            
            # Handle empty path or directory requests
            worker_path = path[len("/pkg/"):]
            if worker_path == "" or worker_path.endswith("/"):
                # Create a JavaScript module that exports a default empty object
                js_content = """
// Minimal module to satisfy worker imports
export default async function init(input) {
  return {};
};

export function wbg_rayon_start_worker(receiver) {
  // Empty implementation
};
"""
                if self.serve_js_module(js_content):
                    return
        
        # Special handling for splats folder
        if path.startswith("/splats/"):
            splat_file = path[len("/splats/"):]
            splat_path = os.path.join(os.getcwd(), "splats", splat_file)
            
            if self.serve_file(splat_path):
                return
        
        # If we get here, the file wasn't found
        return super().do_GET()

def run_server():
    handler = CORSHTTPRequestHandler
    
    # Allow the server to reuse the address
    socketserver.TCPServer.allow_reuse_address = True
    
    with socketserver.TCPServer(("", PORT), handler) as httpd:
        print(f"Serving at http://localhost:{PORT}")
        print("With headers for SharedArrayBuffer support:")
        print("  Cross-Origin-Opener-Policy: same-origin")
        print("  Cross-Origin-Embedder-Policy: require-corp")
        print("  Access-Control-Allow-Origin: *")
        print("  Cross-Origin-Resource-Policy: cross-origin")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nShutting down server...")
            httpd.server_close()

if __name__ == "__main__":
    run_server() 