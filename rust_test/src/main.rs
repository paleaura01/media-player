use log::{debug, error, info, trace, warn};

fn main() {
    // Initialize env_logger. You can control log output via the RUST_LOG env variable.
    env_logger::init();
    
    info!("Test environment initialized.");
    debug!("This is a debug message to verify that debug logging is working.");
    warn!("This is a warning message for testing purposes.");
    error!("This is an error message to check error logging.");
    trace!("This is a trace message (set RUST_LOG=trace to see this).");
    
    println!("Rust is working correctly in the test environment!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_arithmetic() {
        // A simple test to verify that tests run correctly.
        assert_eq!(2 + 2, 4);
    }
}
