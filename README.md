# Rust Web Server with Thread Pool 🦀

A basic multi-threaded web server in Rust, built with a custom thread pool to handle concurrent HTTP requests.

## How to Run

1.  **Prerequisites**: Ensure you have Rust installed.
2.  **Project Setup**:
    *   Create a new Rust project: `cargo new webserver`
    *   Change into the directory: `cd webserver`
    *   Replace `src/main.rs` and `src/lib.rs` with the provided code.
    *   Create `index.html` and `404.html` in the project root (`webserver/`).
        *   **`index.html` example:**
            ```html
            <h1>Hello!</h1><p>Hi from Rust</p>
            ```
        *   **`404.html` example:**
            ```html
            <h1>Woops!</h1><p>Sorry, I don't know what you're asking for.</p>
            ```
3.  **Start Server**: `cargo run`
4.  **Access**: Open your browser to `http://127.0.0.1:7878/`

## Endpoints

*   `/`: Serves `index.html`
*   `/sleep`: Simulates a slow request (5-second delay) then serves `index.html`
*   Any other path: Serves `404.html`

The server will shut down gracefully when stopped (e.g., `Ctrl+C`).