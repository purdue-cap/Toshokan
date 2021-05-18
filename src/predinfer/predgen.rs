use std::collections::{HashMap, HashSet};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GrammarInput {
    pub content: HashMap<String, HashSet<Vec<String>>>,
    pub start_symbol: String,
}

pub struct Grammar {
    content: HashMap<String, HashSet<Vec<String>>>,
    start_symbol: String,
    terminals: HashSet<String>,
    non_terminals: HashSet<String>
}

#[derive(Debug)]
pub struct Node {
    pub data: Option<String>,
    pub children: Vec<Node>
}

pub struct PredGenerator<'g> {
    grammar: &'g Grammar
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

    pub fn from_input(input: GrammarInput) -> Self{
        Self::from_content(input.content, input.start_symbol)
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

    pub fn get_terminating_productions(&self, symbol: &str) -> Vec<&Vec<String>> {
        if let Some(prods) = self.content.get(symbol) {
            prods.iter().filter(|prod| prod.iter().all(|rhs| self.get_terminals().contains(rhs))).collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_non_terminating_productions(&self, symbol: &str) -> Vec<&Vec<String>> {
        if let Some(prods) = self.content.get(symbol) {
            prods.iter().filter(|prod| !prod.iter().all(|rhs| self.get_terminals().contains(rhs))).collect()
        } else {
            Vec::new()
        }
    }
}

impl Node {
    pub fn to_string(&self) -> String {
        if self.children.is_empty() {
            self.data.as_ref().unwrap_or(&"".to_string()).clone()
        } else {
            let mut formatted = self.children.iter().map(|child| {
                let child_string = child.to_string();
                if let Some(last_char) = child_string.chars().last() {
                    if last_char.is_whitespace() {
                        return child_string;
                    }
                }
                format!("{} ", child_string)
            })
                .collect::<Vec<String>>().join("");
            if let Some(last_char) = formatted.chars().last() {
                if last_char == ' ' {
                    formatted.pop();
                }
            }
            formatted
        }
    }
}

impl<'g> PredGenerator<'g> {
    pub fn new(grammar: &'g Grammar) -> Self {
        Self {
            grammar: grammar
        }
    }

    pub fn generate_random_ast_for_symbol<S: AsRef<str>>(&self, height: usize, symbol: S) -> Option<Node> {
        let mut rng = thread_rng();
        if height > 0 {
            let prods = self.grammar.get_non_terminating_productions(symbol.as_ref());
            let prod = prods.choose(&mut rng)?;
            Some(Node {
                data: None,
                children: prod.iter().map(|target| {
                    if self.grammar.get_terminals().contains(target) {
                        Some(Node {data:Some(target.clone()), children:vec![]})
                    } else {
                        self.generate_random_ast_for_symbol(height - 1, target)
                    }
                }).collect::<Option<Vec<_>>>()?
            })
        } else {
            let prods = self.grammar.get_terminating_productions(symbol.as_ref());
            let prod = prods.choose(&mut rng)?;
            Some(Node {
                data: None,
                children: prod.iter().cloned().map(|data| Node{data:Some(data), children:vec![]}).collect()
            })
        }
    }

    pub fn generate_random_full_ast(&self, height: usize) -> Option<Node> {
        self.generate_random_ast_for_symbol(height, self.grammar.get_start_symbol())
    }
}

#[cfg(test)]
mod tests {
    use super::Grammar;
    use super::PredGenerator;
    use std::error::Error;
    use std::collections::HashSet;
    use std::collections::HashMap;

    fn get_content() -> HashMap<String, HashSet<Vec<String>>> {
        vec![
            ("A".to_string(), vec![vec!["a".to_string()], vec!["a".to_string(), "B".to_string()]].into_iter().collect()), // A := a | a B
            ("B".to_string(), vec![vec!["b".to_string()], vec!["b".to_string(), "c".to_string()]].into_iter().collect()) // B := b | b c
        ].into_iter().collect()
    }

    fn get_complex_content() -> HashMap<String, HashSet<Vec<String>>> {
        vec![
            ("A".to_string(), vec![vec!["a".to_string()], vec!["a".to_string(), "B".to_string()]].into_iter().collect()), // A := a | a B
            ("B".to_string(), vec![vec!["b".to_string()], vec!["A".to_string(), "c".to_string()]].into_iter().collect()) // B := b | A c
        ].into_iter().collect()
    }

    #[test]
    fn grammar_initialization() -> Result<(), Box<dyn Error>> {
        let grammar = Grammar::from_content(get_content(), "A".to_string());
        assert_eq!(grammar.get_terminals(), &vec!["a".to_string(), "b".to_string(), "c".to_string()].into_iter().collect::<HashSet<_>>());
        assert_eq!(grammar.get_non_terminals(), &vec!["A".to_string(), "B".to_string()].into_iter().collect::<HashSet<_>>());
        Ok(())
    }

    static GRAMMAR_STR: &'static str =
r#"start_symbol: Start
content:
  Start:
    - [A, B]
    - [B, A]
  A:
    - [a]
    - [a, A]
  B:
    - [b]
    - [b, A]
"#;

    #[test]
    fn grammar_from_yaml() -> Result<(), Box<dyn Error>> {
        let grammar = Grammar::from_input(serde_yaml::from_str(GRAMMAR_STR)?);
        assert_eq!(grammar.get_terminals(), &vec!["a".to_string(), "b".to_string()].into_iter().collect::<HashSet<_>>());
        assert_eq!(grammar.get_non_terminals(), &vec!["Start".to_string(), "A".to_string(), "B".to_string()].into_iter().collect::<HashSet<_>>());
        Ok(())
    }

    #[test]
    fn get_productions() -> Result<(), Box<dyn Error>> {
        let grammar = Grammar::from_content(get_content(), "A".to_string());
        assert_eq!(grammar.get_terminating_productions("A"), vec![&vec!["a".to_string()]]);
        assert_eq!(grammar.get_terminating_productions("B"),
            vec![&vec!["b".to_string()], &vec!["b".to_string(), "c".to_string()]]);
        assert_eq!(grammar.get_non_terminating_productions("A"), vec![&vec!["a".to_string(), "B".to_string()]]);
        assert!(grammar.get_non_terminating_productions("B").is_empty());
        Ok(())
    }

    #[test]
    fn rand_gen_asts() -> Result<(), Box<dyn Error>> {
        let grammar = Grammar::from_content(get_complex_content(), "A".to_string());
        let gen = PredGenerator::new(&grammar);
        println!("{}", gen.generate_random_full_ast(5).ok_or("Generation Failure")?.to_string());
        println!("{}", gen.generate_random_full_ast(4).ok_or("Generation Failure")?.to_string());
        println!("{}", gen.generate_random_full_ast(3).ok_or("Generation Failure")?.to_string());
        println!("{}", gen.generate_random_full_ast(2).ok_or("Generation Failure")?.to_string());
        Ok(())
    }
}