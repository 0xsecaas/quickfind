# quickfind

**Search files instantly: configurable, interactive, Rust-powered.**

Remember part of a filename? Find it instantly in milliseconds, open it in your default app or jump straight into `vim`.

## Install Quickly

```bash
$ cargo install quickfind
```

<details> <summary>Usage</summary>

## 1. Index once

```bash
$ quickfind index
```

## 2. Search any moment

```bash
$ quickfind <your-query>

# OR

$ quickfind
```
</details> 

---


<details> <summary>Why quickfind?</summary>

Since I started using Linux, I always felt one essential tool was missing: a fast, reliable file finder like _Everything Search_ on Windows.  
So I built **quickfind** in Rust. Its configurable indexing and interactive TUI make finding files fast, reliable, and effortless.

</details>

<details> <summary>Features</summary>

- **Configurable:** Customize search locations, ignored paths, and search depth via a simple config file.
- **Efficient Indexing:** Traverses directories once and stores paths in a local database for lightning-fast searching.
- **Interactive Interface:** Browse results with a minimal TUI, open files in default apps or `vim`.

</details>

<details> <summary>Install from Source</summary>
1. Clone the repository:

```bash
$ git clone https://github.com/0xsecaas/quickfind
```

2. Build the project:

```bash
$ cd quickfind
$ cargo build --release
```

3. Run the application:

```bash
$ ./target/release/quickfind

# OR

$ cargo run 
```

</details> 

<details> <summary>Configuration</summary>

Config file: `~/.quickfind/config.toml`

```toml
include = [
    "/path/to/your/directory",
    "/another/path/to/search"
]
ignore = "**/node_modules/**"
depth = 10
editor = "vim" # "vi" or "code" or "subl" or any editor of your choice
```

- `include`: Absolute paths to directories you want to index.
- `ignore`: Glob patterns for paths to exclude.
- `depth`: Maximum directory depth to traverse.
</details> 

<details> <summary>Interactive Mode</summary>

- `Tab`: Switch between search input and results
- `Arrow Keys`: Navigate results
- `Enter`: Open selected file/directory with default app
- `v`: Open selected file with vim
- `d`: Open containing directory
- `Esc`: Exit interactive mode

</details> 

<details> <summary>Architecture</summary>

- `main.rs`: CLI parsing and orchestration
- `config.rs`: Loads and manages user configs (~/.quickfind/config.toml)
- `db.rs`: Handles persistent file indexing storage
- `indexing.rs`: Traverses directories and populates the database
- `tui.rs`: Interactive Text User Interface

</details> 

<details> <summary>Future Plans</summary>

- **Background Sync**: Automatically update the index as files change

</details> 

<details> <summary>Contributing</summary>

Open issues, submit PRs, or suggest features.

</details> 

<details> <summary>License</summary>

MIT License

</details>
