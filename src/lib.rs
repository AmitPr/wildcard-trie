//! # Radix Trie for URL Routing
//!
//! A space-efficient trie that compresses paths by storing common prefixes in single nodes.
//! This prevents DoS attacks from extremely long segmented paths while maintaining fast lookups.
//!
//! This crate supports:
//! - Wildcard Support: Routes ending in `/*` match any sub-path  
//! - Fast Lookups: `O(path_length)`` instead of `O(number_of_routes)`
//! - DoS Resistant: Long paths don't create excessive nodes
//! - Compressed representation: `/api/v1/users` and `/api/v1/posts` share the `/api/v1/` prefix
//!
//! ## Example
//! ```rust
//! use wildcard_trie::Trie;
//!
//! let mut trie = Trie::new();
//! trie.insert("/api/*", "api_handler");           // Wildcard
//! trie.insert("/api/users", "users_handler");     // Exact (takes precedence)
//!
//! assert_eq!(trie.get("/api/users"), Some(&"users_handler"));  // Exact match
//! assert_eq!(trie.get("/api/posts"), Some(&"api_handler"));    // Wildcard match
//! ```

#[cfg(feature = "debug")]
mod prettyprint;

use std::collections::HashMap;

/// Suffix that indicates a wildcard route (matches any sub-path)
const WILDCARD_SUFFIX: &str = "/*";

/// A node in the radix trie that stores a compressed path prefix
#[derive(Debug, Clone)]
struct RadixNode<T> {
    /// The path prefix stored at this node (e.g., "/api/v1")
    prefix: String,
    /// Child nodes, indexed by the first character of their prefix
    children: HashMap<char, RadixNode<T>>,
    /// Value for exact path matches at this node
    exact_value: Option<T>,
    /// Value for wildcard matches (/*) at this node
    wildcard_value: Option<T>,
}

impl<T> RadixNode<T> {
    /// Creates a new node with the given prefix
    fn new(prefix: String) -> Self {
        Self {
            prefix,
            children: HashMap::new(),
            exact_value: None,
            wildcard_value: None,
        }
    }

    /// Inserts a value at the given path
    fn insert(&mut self, path: &str, value: T, is_wildcard: bool) {
        if path.is_empty() {
            self.store_value(value, is_wildcard);
            return;
        }

        let common_length = self.count_common_prefix_chars(path);

        // Split this node if the path diverges from our prefix
        if common_length < self.prefix.len() {
            self.split_at(common_length);
        }

        // Continue to child or store at current node
        if common_length < path.len() {
            self.insert_in_child(&path[common_length..], value, is_wildcard);
        } else {
            self.store_value(value, is_wildcard);
        }
    }

    /// Retrieves a value for the given path, considering wildcards
    fn get(&self, path: &str) -> Option<&T> {
        self.get_with_fallback(path, None)
    }

    /// Removes a value at the given path
    fn remove(&mut self, path: &str, is_wildcard: bool) -> Option<T> {
        if path.is_empty() {
            return self.take_value(is_wildcard);
        }

        let common_length = self.count_common_prefix_chars(path);
        if common_length != self.prefix.len() {
            return None; // Path doesn't exist
        }

        let remaining_path = &path[common_length..];
        if remaining_path.is_empty() {
            self.take_value(is_wildcard)
        } else {
            self.remove_from_child(remaining_path, is_wildcard)
        }
    }

    /// Stores a value in the appropriate slot (exact or wildcard)
    fn store_value(&mut self, value: T, is_wildcard: bool) {
        if is_wildcard {
            self.wildcard_value = Some(value);
        } else {
            self.exact_value = Some(value);
        }
    }

    /// Takes a value from the appropriate slot (exact or wildcard)
    fn take_value(&mut self, is_wildcard: bool) -> Option<T> {
        if is_wildcard {
            self.wildcard_value.take()
        } else {
            self.exact_value.take()
        }
    }

    /// Counts how many characters this node's prefix shares with the given path
    fn count_common_prefix_chars(&self, path: &str) -> usize {
        self.prefix
            .chars()
            .zip(path.chars())
            .take_while(|(a, b)| a == b)
            .count()
    }

    /// Retrieves value with wildcard fallback support
    fn get_with_fallback<'a>(&'a self, path: &str, fallback: Option<&'a T>) -> Option<&'a T> {
        // Update fallback if we have a wildcard at this level
        let current_fallback = self.wildcard_value.as_ref().or(fallback);

        if path.is_empty() {
            return self
                .exact_value
                .as_ref()
                .or(self.wildcard_value.as_ref())
                .or(fallback);
        }

        let common_length = self.count_common_prefix_chars(path);

        if common_length == self.prefix.len() {
            let remaining_path = &path[common_length..];

            if remaining_path.is_empty() {
                // Exact match at this node
                self.exact_value
                    .as_ref()
                    .or(self.wildcard_value.as_ref())
                    .or(current_fallback)
            } else {
                // Continue searching in children
                self.search_in_child(remaining_path, current_fallback)
            }
        } else {
            // Partial match - return original fallback, not our wildcard
            fallback
        }
    }

    /// Inserts value in the appropriate child node
    fn insert_in_child(&mut self, remaining_path: &str, value: T, is_wildcard: bool) {
        let first_char = remaining_path.chars().next().unwrap();
        self.children
            .entry(first_char)
            .or_insert_with(|| RadixNode::new(remaining_path.to_string()))
            .insert(remaining_path, value, is_wildcard);
    }

    /// Searches for a value in child nodes
    fn search_in_child<'a>(
        &'a self,
        remaining_path: &str,
        fallback: Option<&'a T>,
    ) -> Option<&'a T> {
        let first_char = remaining_path.chars().next().unwrap();
        if let Some(child) = self.children.get(&first_char) {
            child.get_with_fallback(remaining_path, fallback)
        } else {
            fallback
        }
    }

    /// Removes value from the appropriate child node
    fn remove_from_child(&mut self, remaining_path: &str, is_wildcard: bool) -> Option<T> {
        let first_char = remaining_path.chars().next().unwrap();
        if let Some(child) = self.children.get_mut(&first_char) {
            child.remove(remaining_path, is_wildcard)
        } else {
            None
        }
    }

    /// Splits this node at the given position to accommodate path divergence
    fn split_at(&mut self, split_position: usize) {
        if split_position >= self.prefix.len() {
            return;
        }

        // Create new child with the suffix
        let suffix = self.prefix.split_off(split_position);
        let mut new_child = RadixNode::new(suffix.clone());

        // Move our data to the new child
        new_child.children = std::mem::take(&mut self.children);
        new_child.exact_value = self.exact_value.take();
        new_child.wildcard_value = self.wildcard_value.take();

        // Add the new child
        let first_char = suffix.chars().next().unwrap();
        self.children.insert(first_char, new_child);
    }
}

/// A radix trie for efficient path-based routing with wildcard support
#[derive(Debug)]
pub struct Trie<T>(RadixNode<T>);

impl<T> Default for Trie<T> {
    fn default() -> Self {
        Self(RadixNode::new(String::new()))
    }
}

impl<T> Trie<T> {
    /// Creates a new empty trie
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a value at the given path
    ///
    /// Paths ending with `/*` are treated as wildcard routes that match any sub-path.
    ///
    /// # Examples
    /// ```rust
    /// # use wildcard_trie::Trie;
    /// let mut trie = Trie::new();
    /// trie.insert("/api/users", "users_handler");
    /// trie.insert("/api/*", "api_fallback");
    /// ```
    pub fn insert(&mut self, path: &str, value: T) {
        let (clean_path, is_wildcard) = Self::parse_path(path);
        self.0.insert(clean_path, value, is_wildcard);
    }

    /// Retrieves a value for the given path, with exact > wildcard precedence.
    ///
    /// # Examples
    /// ```rust
    /// # use wildcard_trie::Trie;
    /// # let mut trie = Trie::new();
    /// # trie.insert("/api/users", "users_handler");
    /// # trie.insert("/api/*", "api_fallback");
    /// assert_eq!(trie.get("/api/users"), Some(&"users_handler"));  // Exact
    /// assert_eq!(trie.get("/api/posts"), Some(&"api_fallback"));   // Wildcard
    /// ```
    pub fn get<'a>(&'a self, path: &str) -> Option<&'a T> {
        self.0.get(path)
    }

    /// Removes a value at the given path, returning it if it existed
    pub fn remove(&mut self, path: &str) -> Option<T> {
        let (clean_path, is_wildcard) = Self::parse_path(path);
        self.0.remove(clean_path, is_wildcard)
    }

    /// Parses a path to determine if it's a wildcard and extract the clean path
    fn parse_path(path: &str) -> (&str, bool) {
        if let Some(prefix) = path.strip_suffix(WILDCARD_SUFFIX) {
            (prefix, true)
        } else {
            (path, false)
        }
    }

    /// Checks if the trie is empty
    fn is_empty(&self) -> bool {
        self.0.children.is_empty()
            && self.0.exact_value.is_none()
            && self.0.wildcard_value.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_path_matching() {
        let mut trie = Trie::new();
        trie.insert("/api/users", "users_handler");
        trie.insert("/api/posts", "posts_handler");

        assert_eq!(trie.get("/api/users"), Some(&"users_handler"));
        assert_eq!(trie.get("/api/posts"), Some(&"posts_handler"));
        assert_eq!(trie.get("/api/other"), None);
    }

    #[test]
    fn test_wildcard_matching() {
        let mut trie = Trie::new();
        trie.insert("/api/*", "api_handler");

        assert_eq!(trie.get("/api/users"), Some(&"api_handler"));
        assert_eq!(trie.get("/api/posts/123"), Some(&"api_handler"));
        assert_eq!(trie.get("/auth/login"), None);
    }

    #[test]
    fn test_exact_takes_precedence_over_wildcard() {
        let mut trie = Trie::new();
        trie.insert("/api/*", "wildcard_handler");
        trie.insert("/api/users", "exact_handler");

        assert_eq!(trie.get("/api/users"), Some(&"exact_handler"));
        assert_eq!(trie.get("/api/posts"), Some(&"wildcard_handler"));
    }

    #[test]
    fn test_path_compression() {
        let mut trie = Trie::new();
        trie.insert("/api/v1/users", "v1_users");
        trie.insert("/api/v1/posts", "v1_posts");
        trie.insert("/api/v2/users", "v2_users");

        assert_eq!(trie.get("/api/v1/users"), Some(&"v1_users"));
        assert_eq!(trie.get("/api/v1/posts"), Some(&"v1_posts"));
        assert_eq!(trie.get("/api/v2/users"), Some(&"v2_users"));
    }

    #[test]
    fn test_removal() {
        let mut trie = Trie::new();
        trie.insert("/api/users", "handler");

        assert_eq!(trie.get("/api/users"), Some(&"handler"));
        assert_eq!(trie.remove("/api/users"), Some("handler"));
        assert_eq!(trie.get("/api/users"), None);
    }

    #[test]
    fn test_wildcard_removal() {
        let mut trie = Trie::new();
        trie.insert("/api/*", "handler");

        assert_eq!(trie.get("/api/users"), Some(&"handler"));
        assert_eq!(trie.remove("/api/*"), Some("handler"));
        assert_eq!(trie.get("/api/users"), None);
    }

    #[test]
    fn test_root_path() {
        let mut trie = Trie::new();
        trie.insert("/", "root_handler");
        assert_eq!(trie.get("/"), Some(&"root_handler"));
    }

    #[test]
    fn test_root_wildcard() {
        let mut trie = Trie::new();
        trie.insert("/*", "root_handler");
        assert_eq!(trie.get("/"), Some(&"root_handler"));
    }

    #[test]
    fn test_empty_path() {
        let mut trie = Trie::new();
        trie.insert("", "empty_handler");
        assert_eq!(trie.get(""), Some(&"empty_handler"));
    }

    #[test]
    fn test_common_prefix() {
        let mut trie = Trie::new();
        trie.insert("long_prefix_one", "one");
        trie.insert("long_prefix_two", "two");
        trie.insert("long_prefix_three", "three");

        assert_eq!(trie.get("long_prefix_one"), Some(&"one"));
        assert_eq!(trie.get("long_prefix_two"), Some(&"two"));
        assert_eq!(trie.get("long_prefix_three"), Some(&"three"));
    }
}
