use std::collections::{HashMap, HashSet};

pub struct Grammar {
    content: HashMap<String, HashSet<Vec<String>>>,
    terminals: HashSet<String>,
    non_terminals: HashSet<String>,
    start_symbol: String
}

pub struct Node {
    pub data: String,
    pub children: Vec<Node>
}

impl Grammar {
    pub fn from_content(content: HashMap<String, HashSet<Vec<String>>>, start: String) -> Self {
        let non_terminals : HashSet<&String> = content.keys().collect();
        let all_rh_symbols : HashSet<&String> = content.values().flatten().flatten().collect();
        Self {
            terminals: all_rh_symbols.difference(&non_terminals).cloned().cloned().collect(),
            non_terminals: non_terminals.into_iter().cloned().collect(),
            content: content,
            start_symbol: start
        }
    }

    pub fn get_terminals(&self) -> &HashSet<String> {
        &self.terminals
    }
    pub fn get_non_terminals(&self) -> &HashSet<String> {
        &self.non_terminals
    }

    pub fn get_start_symbol(&self) -> &String {
        &self.start_symbol
    }

    pub fn get_terminating_productions(&self, symbol: &str) -> HashSet<&Vec<String>> {
        if let Some(prods) = self.content.get(symbol) {
            prods.iter().filter(|prod| prod.iter().all(|rhs| self.get_terminals().contains(rhs))).collect()
        } else {
            HashSet::new()
        }
    }

    pub fn get_non_terminating_productions(&self, symbol: &str) -> HashSet<&Vec<String>> {
        if let Some(prods) = self.content.get(symbol) {
            prods.iter().filter(|prod| !prod.iter().all(|rhs| self.get_terminals().contains(rhs))).collect()
        } else {
            HashSet::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Grammar;
    use std::error::Error;
    use std::collections::HashSet;
    use std::collections::HashMap;

    fn get_content() -> HashMap<String, HashSet<Vec<String>>> {
        vec![
            ("A".to_string(), vec![vec!["a".to_string()], vec!["a".to_string(), "B".to_string()]].into_iter().collect()), // A := a | a B
            ("B".to_string(), vec![vec!["b".to_string()], vec!["b".to_string(), "c".to_string()]].into_iter().collect()) // B := b | b c
        ].into_iter().collect()
    }

    #[test]
    fn grammar_initialization() -> Result<(), Box<dyn Error>> {
        let grammar = Grammar::from_content(get_content(), "A".to_string());
        assert_eq!(grammar.get_terminals(), &vec!["a".to_string(), "b".to_string(), "c".to_string()].into_iter().collect::<HashSet<_>>());
        assert_eq!(grammar.get_non_terminals(), &vec!["A".to_string(), "B".to_string()].into_iter().collect::<HashSet<_>>());
        Ok(())
    }

    #[test]
    fn get_productions() -> Result<(), Box<dyn Error>>  {
        let grammar = Grammar::from_content(get_content(), "A".to_string());
        assert_eq!(grammar.get_terminating_productions("A"), vec![&vec!["a".to_string()]].into_iter().collect::<HashSet<_>>());
        assert_eq!(grammar.get_terminating_productions("B"),
            vec![&vec!["b".to_string()], &vec!["b".to_string(), "c".to_string()]].into_iter().collect::<HashSet<_>>());
        assert_eq!(grammar.get_non_terminating_productions("A"), vec![&vec!["a".to_string(), "B".to_string()]].into_iter().collect::<HashSet<_>>());
        assert_eq!(grammar.get_non_terminating_productions("B"), vec![].into_iter().collect::<HashSet<_>>());
        Ok(())
    }
}