// importScripts("./pkg/wasm_in_web_worker.js");
// import { NumberEval } from "./wasm_in_web_worker.js";

// worker.js - Handles WebAssembly initialization and thread pool setup
import init, { initThreadPool } from "./pkg/gs_rust.js";

// Listen for messages from the main thread
self.onmessage = async function (e) {
  const { type, numThreads } = e.data;

  if (type === "init") {
    try {
      // Initialize the WebAssembly module
      self.postMessage({ status: "initializing_wasm" });
      await init();
      self.postMessage({ status: "wasm_initialized" });

      // Initialize the thread pool with the specified number of threads
      self.postMessage({
        status: "initializing_thread_pool",
        threads: numThreads,
      });
      await initThreadPool(numThreads);
      self.postMessage({ status: "thread_pool_initialized", success: true });
    } catch (error) {
      self.postMessage({
        status: "error",
        message: error.toString(),
        stack: error.stack,
      });
    }
  } else if (type === "execute") {
    // Here you would add code to execute your Rust functions
    // For example: const result = yourRustFunction(e.data.params);
    // self.postMessage({ status: 'result', data: result });
    self.postMessage({ status: "ready_for_commands" });
  }
};

// Notify the main thread that the worker is ready
self.postMessage({ status: "worker_loaded" });

console.log("Initializing worker");

// Use a try-catch block to handle potential import errors gracefully
try {
  // Import the WASM module
  import("./pkg/gs_rust.js")
    .then((module) => {
      const { testing } = module;
      console.log("Worker module loaded successfully");

      // Set callback to handle messages passed to the worker
      self.onmessage = async (event) => {
        console.log("Worker received message:", event.data);

        try {
          // Call the testing function from the WASM module if available
          if (typeof testing === "function") {
            testing(3);
          }

          // Send response back to be handled by callback in main thread
          self.postMessage({
            type: "result",
            message: "Worker processed the message successfully",
          });
        } catch (error) {
          console.error("Error in worker processing:", error);
          self.postMessage({
            type: "error",
            message: error.toString(),
          });
        }
      };

      // Signal that the worker is ready
      self.postMessage({
        type: "ready",
        message: "Worker initialized and ready",
      });
    })
    .catch((error) => {
      console.error("Failed to load worker module:", error);
      // Notify the main thread about the error
      self.postMessage({
        type: "error",
        message: "Failed to initialize worker: " + error.toString(),
      });
    });
} catch (error) {
  console.error("Critical error in worker:", error);
}

console.log("Worker initialization process started");
