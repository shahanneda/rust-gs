// This is a simple worker that responds to messages from the main thread
// and imports the wasm module when needed

// Set up error handling for uncaught exceptions
self.addEventListener("error", function (e) {
  console.error("Worker error:", e.message);
  self.postMessage({
    type: "error",
    error: `Uncaught error in worker: ${e.message}`,
  });
});

// Set up unhandled rejection handling
self.addEventListener("unhandledrejection", function (e) {
  console.error("Worker unhandled promise rejection:", e.reason);
  self.postMessage({
    type: "error",
    error: `Unhandled promise rejection in worker: ${e.reason}`,
  });
});

// Handle messages from the main thread
self.onmessage = async function (e) {
  const msg = e.data;
  console.log(`Worker received message of type: ${msg.type}`);

  // Respond immediately to initialization messages
  if (msg.type === "wasm_bindgen_worker_init") {
    console.log("Worker received init message, responding immediately");
    // Respond immediately to let the main thread know we're alive
    self.postMessage({ type: "wasm_bindgen_worker_ack" });

    try {
      // The actual initialization will happen asynchronously
      const { init, receiver } = msg;
      console.log("Worker starting initialization");

      // Import the wasm module
      const pkg = await import("../../..");
      console.log("Worker imported module");

      // Initialize the module
      await pkg.default(init.module_or_path);
      console.log("Worker initialized module");

      // Signal that we're ready
      self.postMessage({ type: "wasm_bindgen_worker_ready" });
      console.log("Worker sent ready message");

      // Start the worker
      pkg.wbg_rayon_start_worker(receiver);
      console.log("Worker started rayon worker");
    } catch (error) {
      console.error("Worker initialization error:", error);
      self.postMessage({
        type: "error",
        error: `Worker initialization failed: ${error.toString()}`,
      });
    }
    return;
  }

  // Handle other message types
  try {
    if (msg.type === "ping") {
      console.log("Worker received ping");
      self.postMessage({ type: "pong" });
    } else {
      console.log("Worker received unknown message type:", msg.type);
      self.postMessage({
        type: "error",
        error: `Unknown message type: ${msg.type}`,
      });
    }
  } catch (error) {
    console.error("Worker error handling message:", error);
    self.postMessage({
      type: "error",
      error: `Error handling message: ${error.toString()}`,
    });
  }
};

// Signal that the worker is ready to receive messages
console.log("Worker initialized and ready to receive messages");
self.postMessage({ type: "worker_initialized" });
