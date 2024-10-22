#[cfg(target_arch = "wasm32")]
use web_sys::console;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

const DISABLED: bool = true;
pub struct Timer<'a> {
    name: &'a str,
    #[cfg(not(target_arch = "wasm32"))]
    start: Instant,
    has_ended: bool,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        if DISABLED {
            #[cfg(target_arch = "wasm32")]
            {
                return Timer {
                    name,
                    has_ended: true,
                };
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                return Timer {
                    name,
                    start: Instant::now(),
                    has_ended: false,
                };
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            console::time_with_label(name);
            Timer {
                name,
                has_ended: false,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let start = Instant::now();
            println!("Starting timer: {}", name);
            Timer {
                name,
                start,
                has_ended: false,
            }
        }
    }

    pub fn end(&mut self) {
        if self.has_ended {
            return;
        }
        if DISABLED {
            return;
        }
        self.has_ended = true;
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

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        self.end();
    }
}
