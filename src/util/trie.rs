use std::collections::HashMap;

#[derive(Debug)]
struct TrieNode {
    pub children: HashMap<char, TrieNode>,
    pub is_end_of_word: bool,
}

impl TrieNode {
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            is_end_of_word: false
        }
    }
}

#[derive(Debug)]
pub struct Trie {
    root: TrieNode
}

impl Trie {
    pub fn new() -> Self {
        Self {
            root: TrieNode::new(),
        }
    }

    pub fn insert(&mut self, word: &str) {
        let mut current = &mut self.root;
        for c in word.chars() {
            current = current.children.entry(c).or_insert_with(TrieNode::new);
        }
        current.is_end_of_word = true;
    }

    pub fn find_starting_with(&self, prefix: &str, max: usize) -> Vec<String> {
        let mut results = Vec::new();
        let mut current = &self.root;
        let mut current_word = String::new();

        // Traverse to the end of the prefix
        for char in prefix.chars() {
            match current.children.get(&char) {
                Some(node) => {
                    current = node;
                    current_word.push(char);
                }
                None => return results,
            }
        }

        fn dfs(node: &TrieNode, results: &mut Vec<String>, current_word: &mut String, max: usize) { 
            if results.len() >= max {
                return;
            }
            if node.is_end_of_word {
                results.push(current_word.clone());
            }
            for (&c, child) in &node.children {
                current_word.push(c);
                dfs(child, results, current_word, max);
                current_word.pop();
            }
        }
        dfs(&current, &mut results, &mut current_word, max);

        results
    }
}

#[cfg(test)]
mod tests {
    use super::Trie;

    #[test]
    fn test_insert_and_find() {
        let mut trie = Trie::new();
        trie.insert("apple");
        trie.insert("app");
        trie.insert("past");
        trie.insert("pass");
        trie.insert("part");
        trie.insert("pot");

        let mut result = trie.find_starting_with("app", 10);
        result.sort();
        assert_eq!(result, vec!["app", "apple"]);

        result = trie.find_starting_with("pass", 10);
        result.sort();
        assert_eq!(result, vec!["pass"]);

        result = trie.find_starting_with("pa", 10);
        result.sort();
        assert_eq!(result, vec!["part", "pass", "past"]);
    }

    #[test]
    fn test_find_with_limit() {
        let mut trie = Trie::new();
        trie.insert("apple");
        trie.insert("app");
        trie.insert("apricot");
        trie.insert("`a`````partment");

        let mut result = trie.find_starting_with("ap", 2);
        result.sort();
        assert_eq!(result.len(), 2);

        let mut result = trie.find_starting_with("ap", 3);
        result.sort();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_empty_trie() {
        let trie = Trie::new();
        let result = trie.find_starting_with("a", 10);
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_no_matches() {
        let mut trie = Trie::new();
        trie.insert("apple");
        trie.insert("banana");
        let result = trie.find_starting_with("c", 10);
        assert_eq!(result, Vec::<String>::new());
        let result = trie.find_starting_with("abcdefghijklmnopqrstuvwxyz", 10);
        assert_eq!(result, Vec::<String>::new());
    }
}