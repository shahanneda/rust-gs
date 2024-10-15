#[cfg(target_arch = "wasm32")]
use web_sys::console;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

pub struct Timer<'a> {
    name: &'a str,
    #[cfg(not(target_arch = "wasm32"))]
    start: Instant,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        #[cfg(target_arch = "wasm32")]
        {
            console::time_with_label(name);
            Timer { name }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let start = Instant::now();
            println!("Starting timer: {}", name);
            Timer { name, start }
        }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            console::time_end_with_label(self.name);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let duration = self.start.elapsed();
            println!("Timer '{}' ended after {:?}", self.name, duration);
        }
    }
}
