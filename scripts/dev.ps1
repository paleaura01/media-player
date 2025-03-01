# Clean up old build artifacts (older than 7 days)
cargo sweep -t 7

# Watch for changes and run a workspace check
cargo watch -c -x "check --workspace"
