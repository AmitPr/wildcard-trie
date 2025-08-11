use std::fmt::Debug;

use crate::{RadixNode, Trie};

impl<T: Debug> RadixNode<T> {
    /// Pretty prints the trie structure for debugging
    fn pretty_print(&self, prefix: &str, is_last: bool, is_root: bool) -> String {
        let mut output = String::new();

        // Node connector (except for root)
        if !is_root {
            let connector = if is_last { "└── " } else { "├── " };
            output.push_str(&format!("{prefix}{connector}"));
        }

        // Node label
        if self.prefix.is_empty() && is_root {
            output.push_str("(root)");
        } else {
            output.push_str(&format!("\"{}\"", self.prefix));
        }

        // Node values
        self.append_values_to_output(&mut output);
        output.push('\n');

        // Child nodes
        self.append_children_to_output(&mut output, prefix, is_last, is_root);

        output
    }

    fn append_values_to_output(&self, output: &mut String) {
        let mut values = Vec::new();
        if let Some(ref val) = self.exact_value {
            values.push(format!("exact: {val:?}"));
        }
        if let Some(ref val) = self.wildcard_value {
            values.push(format!("wildcard: {val:?}"));
        }
        if !values.is_empty() {
            output.push_str(&format!(" [{}]", values.join(", ")));
        }
    }

    fn append_children_to_output(
        &self,
        output: &mut String,
        prefix: &str,
        is_last: bool,
        is_root: bool,
    ) {
        let child_prefix = if is_root {
            String::new()
        } else {
            format!("{}{}", prefix, if is_last { "    " } else { "│   " })
        };

        let mut children: Vec<_> = self.children.iter().collect();
        children.sort_by_key(|(c, _)| *c);

        for (i, (_, child)) in children.iter().enumerate() {
            let is_last_child = i == children.len() - 1;
            output.push_str(&child.pretty_print(&child_prefix, is_last_child, false));
        }
    }
}

impl<T: Debug> Trie<T> {
    /// Returns a pretty-printed representation of the trie structure
    pub fn pretty_print(&self) -> String
    where
        T: std::fmt::Debug,
    {
        if self.is_empty() {
            "(empty trie)\n".to_string()
        } else {
            self.0.pretty_print("", true, true)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Trie;

    #[test]
    fn test_pretty_print_showcase() {
        let mut trie = Trie::new();

        trie.insert("/", "home");
        trie.insert("/api/*", "api_fallback");
        trie.insert("/api/v1/users", "users_v1");
        trie.insert("/api/v1/posts", "posts_v1");
        trie.insert("/static/*", "static_files");
        trie.insert("/admin/dashboard", "admin");

        println!("\n🌳 Clean Radix Trie Structure:");
        println!("{}", trie.pretty_print());

        assert!(trie.pretty_print().contains("wildcard"));
    }
}
