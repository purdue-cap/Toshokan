use std::collections::{HashMap, HashSet};
use rand::seq::SliceRandom;
use rand::{thread_rng, seq::IteratorRandom};
use serde::Deserialize;
use itertools::Itertools;

#[derive(Deserialize)]
pub struct GrammarInput {
    pub content: HashMap<String, HashSet<Vec<String>>>,
    pub start_symbol: String,
    #[serde(default)]
    pub height_limit: HashMap<String, usize>,
}

pub struct Grammar {
    content: HashMap<String, HashSet<Vec<String>>>,
    start_symbol: String,
    height_limit: HashMap<String, usize>,
    terminals: HashSet<String>,
    non_terminals: HashSet<String>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Node {
    pub data: Option<String>,
    pub children: Vec<Node>
}

pub struct PredGenerator<'g> {
    grammar: &'g Grammar,
    cache: HashMap<(String, usize), Vec<Node>>,
    cache_symbol_size: HashMap<(String, usize), HashSet<Node>>,
    cache_prod_size: HashMap<(&'g Vec<String>, usize), HashSet<Node>>
}

impl Grammar {
    pub fn from_content(content: HashMap<String, HashSet<Vec<String>>>, start: String,height_limit: HashMap<String, usize>) -> Self {
        
        let non_terminals : HashSet<&String> = content.keys().collect();
        let all_rh_symbols : HashSet<&String> = content.values().flatten().flatten().collect();
        Self {
            terminals: all_rh_symbols.difference(&non_terminals).cloned().cloned().collect(),
            non_terminals: non_terminals.into_iter().cloned().collect(),
            content: content,
            start_symbol: start,
            height_limit: height_limit
        }
    }

    pub fn from_input(input: GrammarInput) -> Self{
        Self::from_content(input.content, input.start_symbol,input.height_limit)
      
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
    pub fn get_height(&self) -> usize {
        if self.children.is_empty() {
            0
        } else {
            1 + self.children.iter().map(|child| child.get_height()).max().expect("Children should not be empty")
        }
    }
    
    pub fn get_size(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|child| child.get_size()).sum::<usize>()
        }
    }

}

// Randomly tries everything in set by calling closure func
// Returns Some(result) if func returns Some(result)
// Returns None if everything in set failed
fn random_try_range<'a, T, F, R>(set: &'a mut HashSet<T>, mut func: F) -> Option<R>
    where F : FnMut(&T) -> Option<R>,
        T: std::cmp::Eq + std::hash::Hash + Clone
    {
    let mut rng = thread_rng();
    while !set.is_empty() {
        let choice = set.iter().choose(&mut rng).expect("Set should not be empty").clone();
        if let Some(result) = func(&choice) {
            return Some(result);
        } else {
            set.remove(&choice);
        }
    }
    None
}

// Returns all possible partitions of {part} numbers so that their collective sum is {sum}
// Each partition is enured to be at least 1, {part} would be considered as 1 if 0 is supplied
// Eg. all_partitions(3, 2) -> [1,2], [2,1]
fn all_partitions(sum: usize, part: usize) -> impl Iterator<Item=Vec<usize>> {
    let comb = (1..sum).combinations( if part == 0 {0} else {part - 1});
    comb.map(move |pos|
        std::iter::once(0).chain(pos.into_iter()).chain(std::iter::once(sum))
        .tuple_windows()
        .map(|(start, end)| end-start)
        .collect_vec()
    )
}

impl<'g> PredGenerator<'g> {
    pub fn new(grammar: &'g Grammar) -> Self {
        Self {
            grammar: grammar,
            cache: HashMap::new(),
            cache_symbol_size: HashMap::new(),
            cache_prod_size: HashMap::new()
        }
    }

    pub fn generate_random_ast_for_symbol<S: AsRef<str>>(&self, mut height: usize, symbol: S) -> Option<Node> {
        let mut rng = thread_rng();
        if self.grammar.height_limit.contains_key(symbol.as_ref()){
            if self.grammar.height_limit.get(symbol.as_ref()).unwrap().clone() < height {
                height = self.grammar.height_limit.get(symbol.as_ref()).unwrap().clone();
            }
        }
        if height > 1 {
            let mut prods : HashSet<_> = self.grammar.get_non_terminating_productions(symbol.as_ref()).into_iter().collect();
            random_try_range(&mut prods, |prod| {
                let mut non_terminal_indexes = prod.iter()
                    .map(|target| self.grammar.get_non_terminals().contains(target))
                    .enumerate()
                    .filter(|(_idx, is_non_terminal)| *is_non_terminal)
                    .map(|(idx, _is_non_terminal)| idx)
                    .collect::<HashSet<_>>();
                let children = random_try_range(&mut non_terminal_indexes, |ensured_height_child|{
                    let mut children = Vec::new();
                    for (i, target) in prod.iter().enumerate() {
                        let child = if i == *ensured_height_child {
                            assert!(self.grammar.get_non_terminals().contains(target));
                            self.generate_random_ast_for_symbol(height - 1, target)?
                        } else {
                            let mut possible_height : HashSet<usize> = (0..height).collect();
                            random_try_range(&mut possible_height, |chosen_height| {
                                self.generate_random_ast_for_symbol(*chosen_height, target)
                            })?
                        };
                        children.push(child);
                    }
                    Some(children)
                })?;
                Some(Node {
                    data: None,
                    children: children
                })
            })
        } else if height == 1 {
            let prods = self.grammar.get_terminating_productions(symbol.as_ref());
            let prod = prods.choose(&mut rng)?;
            Some(Node {
                data: None,
                children: prod.iter().cloned().map(|data| Node{data:Some(data), children:vec![]}).collect()
            })
        } else if self.grammar.get_terminals().contains(symbol.as_ref()) {
            Some(Node {
                data: Some(symbol.as_ref().to_string()),
                children: vec![]
            })
        } else {
            None
        }
    }

    pub fn generate_random_full_ast(&self, height: usize) -> Option<Node> {
        self.generate_random_ast_for_symbol(height, self.grammar.get_start_symbol())
    }

    fn cache_all_ast_for_symbol_with_height<S: AsRef<str>>(&mut self, max_height: usize, symbol: S){
        let query_tuple = (symbol.as_ref().to_string(), max_height);
        if self.cache.contains_key(&query_tuple) {
            return;
        }
        let mut asts = Vec::new();
        if max_height > 1 {
            let prods = self.grammar.get_non_terminating_productions(symbol.as_ref());
            for prod in prods { 
                let mut children_span : Vec<HashSet<Node>> = Vec::new();
                for target in prod {
                    let all_span = (0..max_height).map(|height| {
                        self.cache_all_ast_for_symbol_with_height(height, target.as_str());
                        self.cache.get(&((*target).clone(), height)).expect("Should have ensured cache").clone()
                    }).flatten().collect();
                    children_span.push(all_span);
                }
                let mut ast_for_this_prod = children_span.into_iter()
                    .map(|set| set.into_iter().collect::<Vec<_>>())
                    .multi_cartesian_product()
                    .map(|children|
                    Node{data:None, children:children}
                ).collect();
                asts.append(&mut ast_for_this_prod);
            }
        } else if max_height == 1{
            let prods = self.grammar.get_terminating_productions(symbol.as_ref());
            asts = prods.into_iter().map(|prod|
                Node {data: None, children: prod.iter().cloned().map(|data| Node{data: Some(data), children:vec![]}).collect()})
                .collect();
        } else if self.grammar.get_terminals().contains(symbol.as_ref()) {
            asts.push(Node{
                data: Some(symbol.as_ref().to_string()),
                children: vec![]
            });
        }
        self.cache.insert(query_tuple, asts);
    }

    pub fn generate_all_ast_for_symbol_with_height<S: AsRef<str>>(&mut self, max_height: usize, symbol: S) -> Vec<Node> {
        self.cache_all_ast_for_symbol_with_height(max_height, symbol.as_ref());
        self.cache.get(&(symbol.as_ref().to_string(), max_height)).expect("Should have ensured cache").clone()
    }

    pub fn generate_all_ast_with_height(&mut self, max_height: usize) -> Vec<Node> {
        self.generate_all_ast_for_symbol_with_height(max_height, self.grammar.get_start_symbol())
    }

    fn cache_all_ast_for_prod_with_size<S: AsRef<str>>(&mut self, size: usize, symbol: S, prod: &'g Vec<String>) {
        let query_tuple = (prod, size);
        if self.cache_prod_size.contains_key(&query_tuple) {
            return;
        }
        let mut result = HashSet::new();
        if size == 1 && prod.len() == 0 {
            result.insert(Node {
                data: Some(symbol.as_ref().to_string()),
                children: vec![]
            });
        } else if size > 1 && prod.len() > 0 {
            for parts in all_partitions(size - 1, prod.len()) {
                let nodes = prod.iter().zip(parts.into_iter())
                    .map(|(target, sub_size)| {
                        self.cache_all_ast_for_symbol_with_size(sub_size, &target);
                        self.cache_symbol_size.get(&(target.clone(), sub_size))
                            .expect("Should have ensured cache")
                            .iter().cloned().collect_vec()
                    }).multi_cartesian_product()
                    .map(|children| Node {
                        data: None,
                        children: children
                    });
                result.extend(nodes);
            }
        }
        self.cache_prod_size.insert(query_tuple, result);
    }

    fn cache_all_ast_for_symbol_with_size<S: AsRef<str>>(&mut self, size: usize, symbol: S) {
        let query_tuple = (symbol.as_ref().to_string(), size);
        if self.cache_symbol_size.contains_key(&query_tuple) {
            return;
        }
        if self.grammar.get_non_terminals().contains(symbol.as_ref()) {
            let prods = self.grammar.content.get(symbol.as_ref()).expect("Should have ensured to be non-terminal");
            for prod in prods.iter() {
                self.cache_all_ast_for_prod_with_size(size, symbol.as_ref(), prod);
            }
            let results : HashSet<Node> = prods.iter()
                .map(|prod| self.cache_prod_size.get(&(prod, size)).expect("Should have ensured cache").clone().into_iter()).flatten().collect();
            self.cache_symbol_size.insert(query_tuple, results);
        } else { // Terminal
            if size != 1 { // Terminals could only have size of 1
                self.cache_symbol_size.insert(query_tuple, HashSet::new());
            } else {
                self.cache_symbol_size.insert(query_tuple, vec![
                    Node {
                        data: Some(symbol.as_ref().to_string()),
                        children: vec![]
                    }
                ].into_iter().collect());
            }
        }
    }
    pub fn generate_all_ast_for_symbol_with_size<S: AsRef<str>>(&mut self, size: usize, symbol: S) -> Vec<Node> {
        self.cache_all_ast_for_symbol_with_size(size, symbol.as_ref());
        self.cache_symbol_size.get(&(symbol.as_ref().to_string(), size)).expect("Should have ensured cache").iter().cloned().collect()
    }

    pub fn generate_all_ast_with_size(&mut self, size: usize) -> Vec<Node> {
        self.generate_all_ast_for_symbol_with_size(size, self.grammar.get_start_symbol())
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
        let grammar = Grammar::from_content(get_content(), "A".to_string(), HashMap::new());
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


    macro_rules! vec_string{
        [$($x:tt),*] => {
            vec![$($x),*].into_iter().map(|s| s.to_string()).collect::<Vec<_>>()
        };
    }

    #[test]
    fn get_productions() -> Result<(), Box<dyn Error>> {
        let grammar = Grammar::from_content(get_content(), "A".to_string(), HashMap::new());
        assert_eq!(grammar.get_terminating_productions("A"), vec![&vec!["a".to_string()]]);
        assert_eq!(grammar.get_terminating_productions("B"),
            vec![&vec!["b".to_string()], &vec!["b".to_string(), "c".to_string()]]);
        assert_eq!(grammar.get_non_terminating_productions("A"), vec![&vec!["a".to_string(), "B".to_string()]]);
        assert!(grammar.get_non_terminating_productions("B").is_empty());
        Ok(())
    }

    #[test]
    fn rand_gen_asts() -> Result<(), Box<dyn Error>> {
        let grammar = Grammar::from_content(get_complex_content(), "A".to_string(), HashMap::new());
        let gen = PredGenerator::new(&grammar);
        println!("{}", gen.generate_random_full_ast(5).ok_or("Generation Failure")?.to_string());
        println!("{}", gen.generate_random_full_ast(4).ok_or("Generation Failure")?.to_string());
        println!("{}", gen.generate_random_full_ast(3).ok_or("Generation Failure")?.to_string());
        println!("{}", gen.generate_random_full_ast(2).ok_or("Generation Failure")?.to_string());
        Ok(())
    }
    #[test]
    fn all_gen_asts() -> Result<(), Box<dyn Error>> {
        let grammar = Grammar::from_input(serde_yaml::from_str(GRAMMAR_STR)?);
        let mut gen = PredGenerator::new(&grammar);
        println!("{:?}", gen.generate_all_ast_with_height(5).into_iter().map(|node| node.to_string()).collect::<Vec<_>>());
        println!("{:?}", gen.generate_all_ast_with_height(4).into_iter().map(|node| node.to_string()).collect::<Vec<_>>());
        println!("{:?}", gen.generate_all_ast_with_height(3).into_iter().map(|node| node.to_string()).collect::<Vec<_>>());
        println!("{:?}", gen.generate_all_ast_with_height(2).into_iter().map(|node| node.to_string()).collect::<Vec<_>>());
        Ok(())
    }

    #[test]
    fn all_gen_asts_size() -> Result<(), Box<dyn Error>> {
        let grammar = Grammar::from_input(serde_yaml::from_str(GRAMMAR_STR)?);
        let mut gen = PredGenerator::new(&grammar);
        let mut get_repr = |size: usize| {
            let mut result = gen.generate_all_ast_with_size(size).into_iter().map(|node| node.to_string()).collect::<Vec<_>>();
            result.sort();
            result
        };
        assert_eq!(get_repr(5), vec_string!["a b", "b a"]);
        assert_eq!(get_repr(7), vec_string!["a a b", "a b a", "b a a", "b a a"]);
        assert_eq!(get_repr(9), vec_string!["a a a b", "a a b a", "a b a a", "b a a a", "b a a a", "b a a a"]);
        assert_eq!(get_repr(11), vec_string!["a a a a b", "a a a b a", "a a b a a", "a b a a a", "b a a a a", "b a a a a", "b a a a a", "b a a a a"]);
        Ok(())
    }
}