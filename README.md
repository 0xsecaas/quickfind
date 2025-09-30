# quickfind

**Search files instantly: configurable, interactive**

Since I started using Linux, I always felt one essential tool was missing: a fast, reliable file finder like *Everything Search* on Windows.  
So I built **quickfind** in Rust. Index the directories you care about once, and from then on you only need to remember part of a file name or its extension to locate files instantly. Quickly open files with your default app or jump straight into `vim` inside the terminal. Its configurable indexing, and interactive TUI make finding files fast, reliable, and effortless.

---

## Features

- **Configurable:** Customize search locations, ignored paths, and search depth via a simple configuration file.  
- **Efficient Indexing:** Traverses directories and stores paths in a local database for fast searching.  
- **Interactive Interface:** Browse results with a minimal TUI, open files in default apps or `vim`.  

---

## Install Quickly
```bash
cargo install quickfind
```


## Install Like a Hacker

1. Clone the repository:
```bash
git clone https://github.com/your-username/quickfind.git
```

2. Build the project:
```bash
cd quickfind
cargo build --release
```

3. Run the application:
```bash
./target/release/quickfind
```

## Configuration
The configuration file is located at ~/.quickfind/config.toml.
```toml
include = [
    "/path/to/your/directory",
    "/another/path/to/search"
]
ignore = "**/node_modules/**"
depth = 10

```
- `include`: Absolute paths to directories you want to index.
- `ignore`: Glob patterns for paths to exclude.
- `depth`: Maximum depth to traverse within included directories.

## Usage
### Indexing
Populate the search database with:
```bash
quickfind index
```

## Searching
Search for files interactively:
```bash
quickfind <your-query>
```


## Interactive Mode

- `Tab`: Switch between search input and results.
- `Arrow Keys`: Navigate results.
- `Enter`: Open selected file or directory with default app.
- `v`: Open selected file with `vim`
- `d`: Open the containing directory of the selected file.
- `Esc`: Exit interactive mode.

## Contributing
Contributions are welcome. Submit pull requests or open issues for bugs, feature requests, or suggestions.

## License
MIT License.

---

## Architecture

The `quickfind` application is built in Rust and follows a modular design:

-   **`main.rs`**: The entry point of the application. It parses command-line arguments and orchestrates the execution flow, delegating tasks to other modules.
-   **`config.rs`**: Handles the loading and management of user-defined configurations, typically from `~/.quickfind/config.toml`. This includes settings for directories to include, paths to ignore, and search depth.
-   **`db.rs`**: Manages the persistent storage of file indexing data. It provides an interface for adding, querying, and retrieving file paths from the local database.
-   **`indexing.rs`**: This module is responsible for traversing the file system based on the configuration. It identifies relevant files, filters them according to ignore patterns, and populates the database via `db.rs`.
-   **`tui.rs`**: Implements the interactive Text User Interface (TUI) for searching. It handles user input, displays search results, and provides interactive actions like opening files or directories.

---

## Future Plans

-   **Background Synchronization**: Implement a background service or daemon that continuously monitors specified directories for new files or modifications. This service will automatically update the indexing database without requiring manual intervention, ensuring the search index is always up-to-date.
