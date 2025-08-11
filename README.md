# Wildcard Trie

A space-efficient radix trie implementation for URL routing with wildcard support in Rust. This crate provides fast path lookups with DoS attack resistance by compressing paths and preventing excessive node creation.

## Features

- Wildcard Support: Routes ending in `/*` match any sub-path
- Fast Lookups: `O(path_length)` instead of `O(number_of_routes)`
- DoS Resistant: Long paths don't create excessive nodes due to path compression
- Memory Efficient: Common prefixes are shared (e.g., `/api/v1/users` and `/api/v1/posts` share `/api/v1/`)
- Debug Visualization: Pretty-print trie structure for debugging (with `debug` feature)

## Example

Add this to your `Cargo.toml`:

```toml
[dependencies]
wildcard-trie = "0.1.0"
```

Then use it in your code:

```rust
use wildcard_trie::Trie;

let mut trie = Trie::new();

// Insert routes
trie.insert("/api/*", "api_handler");           // Wildcard route
trie.insert("/api/users", "users_handler");     // Exact route (takes precedence)
trie.insert("/api/v1/posts", "posts_handler");

// Lookup routes
assert_eq!(trie.get("/api/users"), Some(&"users_handler"));  // Exact match
assert_eq!(trie.get("/api/posts"), Some(&"api_handler"));    // Wildcard match
assert_eq!(trie.get("/api/v1/posts"), Some(&"posts_handler"));

// Remove routes
trie.remove("/api/users");
assert_eq!(trie.get("/api/users"), Some(&"api_handler")); // Falls back to wildcard
```

## API Reference

### `Trie<T>`

#### Methods

- `new() -> Self` - Creates an empty trie
- `insert(&mut self, path: &str, value: T)` - Inserts a value at the given path
- `get(&self, path: &str) -> Option<&T>` - Retrieves a value for the path
- `remove(&mut self, path: &str) -> Option<T>` - Removes and returns a value

#### Debug Features

When compiled with the `debug` feature (enabled by default):

- `pretty_print(&self) -> String` - Returns a tree visualization of the trie structure

```rust
let mut trie = Trie::new();
trie.insert("/api/*", "api_handler");
trie.insert("/api/users", "users_handler");

println!("{}", trie.pretty_print());
```

## Examples

### URL Routing

```rust
use wildcard_trie::Trie;

let mut router = Trie::new();

// Set up routes
router.insert("/", "home_page");
router.insert("/api/*", "api_fallback");
router.insert("/api/users", "list_users");
router.insert("/api/users/*", "user_operations");
router.insert("/static/*", "serve_static");

// Route requests
assert_eq!(router.get("/"), Some(&"home_page"));
assert_eq!(router.get("/api/users"), Some(&"list_users"));
assert_eq!(router.get("/api/users/123"), Some(&"user_operations"));
assert_eq!(router.get("/api/posts"), Some(&"api_fallback"));
assert_eq!(router.get("/static/css/main.css"), Some(&"serve_static"));
```

### Path Compression Demonstration

The trie automatically compresses common prefixes:

```rust
let mut trie = Trie::new();

// These routes share the "/api/v1/" prefix
trie.insert("/api/v1/users", "users");
trie.insert("/api/v1/posts", "posts");
trie.insert("/api/v1/comments", "comments");

// Only creates nodes for:
// - "/api/v1/" (shared prefix)
// - "users", "posts", "comments" (suffixes)
```

## Features

### Default Features

- `debug` - Enables pretty-printing functionality

It may be useful to disable the debug feature for code size:

```toml
[dependencies]
wildcard-trie = { version = "0.1.0", default-features = false }
```

## How It Works

The crate uses a radix trie (compressed trie) structure where:

1. Common prefixes are stored in single nodes

   - `/api/v1/users` and `/api/v1/posts` share the `/api/v1/` prefix
   - Prevents DoS attacks from extremely long segmented paths

2. Routes ending with `/*` act as fallbacks

   - Exact matches take precedence over wildcards
   - Wildcards are inherited down the tree for nested matching

3. Each node stores:
   - A compressed path prefix
   - Optional exact match value
   - Optional wildcard match value
   - Child nodes indexed by first character
