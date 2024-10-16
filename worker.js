// importScripts("./pkg/wasm_in_web_worker.js");
// import { NumberEval } from "./wasm_in_web_worker.js";



console.log("Initializing worker");

// // In the worker, we have a different struct that we want to use as in
// // `index.js`.
// importScripts("./pkg/gs_rust.js");
// const { testing } = wasm_bindgen;
import init, {testing} from './pkg/gs_rust.js';

async function init_wasm_in_worker() {
  console.log("INIT WASM IN WORKER");
  // Initialize the WASM module
  // await init();

  // Set callback to handle messages passed to the worker.
  self.onmessage = async (event) => {
    console.log("RECEIVED MESSAGE", event);
    // Call the testing function from the WASM module
    testing(3);
    // Note: num_eval is not defined, so we've commented out this line
    // var worker_result = num_eval.is_even(event.data);

    // Send response back to be handled by callback in main thread.
    // self.postMessage("Worker processed the message");
  };
}

init_wasm_in_worker();
console.log("WORKER");
