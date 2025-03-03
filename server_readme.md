# Python Server for SharedArrayBuffer Support

This simple Python server serves your application with the necessary headers to support SharedArrayBuffer, which is required for your WebAssembly application.

## Why is this needed?

The error message you were seeing:
```
DataCloneError: Failed to execute 'postMessage' on 'Worker': SharedArrayBuffer transfer requires self.crossOriginIsolated.
```

This occurs because browsers require specific security headers to use SharedArrayBuffer:
- `Cross-Origin-Opener-Policy: same-origin`
- `Cross-Origin-Embedder-Policy: require-corp`
- `Cross-Origin-Resource-Policy: cross-origin`

Additionally, the server properly handles CORS with:
- `Access-Control-Allow-Origin: *`
- `Access-Control-Allow-Methods: GET, POST, OPTIONS`
- `Access-Control-Allow-Headers: Content-Type`

## How to use

1. Make sure you have Python 3 installed
2. Run the server:
   ```
   python3 server.py
   ```
   Or use the provided shell script:
   ```
   ./run_server.sh
   ```
3. Open your browser and navigate to:
   ```
   http://localhost:5502
   ```

The server will:
- Serve your application on port 5502
- Add the required headers for SharedArrayBuffer support
- Properly serve files from the splats folder
- Handle CORS requests correctly
- Provide proper MIME types for JavaScript modules and WebAssembly files

## Memory Error

If you encounter a memory error like this:
```
RuntimeError: unreachable
at gs_rust.wasm.std::alloc::rust_oom
```

This is because the WebAssembly application is trying to allocate more memory than is available. This can happen when loading large splat files. You might need to:

1. Try a smaller splat file
2. Increase the memory limit in your WebAssembly configuration
3. Optimize the application to use less memory

## Troubleshooting

If you encounter any issues:
- Make sure no other application is using port 5502
- Check that Python 3 is installed and in your PATH
- Ensure you're running the server from the root directory of your project
- If you're still having issues with SharedArrayBuffer, try using a different browser (Chrome is recommended)
- Check the browser console for specific error messages 