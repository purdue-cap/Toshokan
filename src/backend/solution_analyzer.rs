use clang::Index;
use clang::TranslationUnit;
use clang::Parser;
use clang::Unsaved;
use clang::Entity;
use clang::EntityKind;

pub fn find_first_match<'i, F: Fn(&Entity) -> bool>(e:&Entity<'i>, matcher:&F) -> Option<Entity<'i>> {
    let children  = e.get_children();
    for child in children.into_iter() {
        if matcher(&child) {
            return Some(child);
        }
        if let Some(result) = find_first_match(&child, matcher) {
            return Some(result);
        } 
    }
    None
}

pub fn find_all_matches<'i, F: Fn(&Entity) -> bool>(entity:&Entity<'i>, matcher:&F) -> Vec<Entity<'i>> {
    let mut result = Vec::<Entity<'i>>::new();

    fn recursive_find<'i, F: Fn(&Entity) -> bool>(e:&Entity<'i>, m:&F, v:&mut Vec<Entity<'i>>) {
        let children = e.get_children();
        for child in children.into_iter() {
            if m(&child) {
                v.push(child)
            }
            recursive_find(&child, m, v);
        }
    }
    
    recursive_find(entity, matcher, &mut result);
    result
}

pub struct SolutionAnalyzer<'i> {
    index: &'i Index<'i>,
    parser: Option<Parser<'i>>,
    tu: Option<TranslationUnit<'i>>
}

impl<'i> SolutionAnalyzer<'i> {
    pub fn new(index: &'i Index) -> SolutionAnalyzer<'i> {
        SolutionAnalyzer {
            index: index,
            parser: None,
            tu: None
        }
    }
    
    pub fn parse_string<S: AsRef<str>>(&mut self, content:S) -> &mut Self {
        let src_files = [Unsaved::new("tmp_src.cpp", content),];
        self.parser = Some(self.index.parser("tmp_src.cpp"));
        self.tu = if let Some(ref mut p) = self.parser.as_mut() {
            p.unsaved(&src_files).parse().ok()
        } else {None};
        self
    }

    pub fn get_root_entity(&self) -> Option<Entity> {
        Some(self.tu.as_ref()?.get_entity())
    }

    pub fn find_first_match<F: Fn(&Entity) -> bool>(&self, matcher:&F) -> Option<Entity> {
        find_first_match(self.get_root_entity().as_ref()?, matcher)
    }

    pub fn find_all_matches<F: Fn(&Entity) -> bool>(&self, matcher:&F) -> Vec<Entity> {
        if let Some(ref entity) = self.get_root_entity() {
            find_all_matches(entity, matcher)
        } else {Vec::<Entity<'i>>::new()}
    }

    pub fn find_func_def<S: AsRef<str>>(&self, target:S) -> Option<Entity> {
        let func_matcher = |e:&Entity| -> bool {
            e.is_definition() &&
            e.get_kind() == EntityKind::FunctionDecl &&
            if let Some(name) = e.get_name() {
                name == target.as_ref()
            } else {false}
        };
        self.find_first_match(&func_matcher)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clang::Clang;
    use clang::TypeKind;

    macro_rules! init {
        ($analyzer:ident) => {
            let cl = Clang::new().unwrap();
            let ind = Index::new(&cl, false, false);
            let mut $analyzer = SolutionAnalyzer::new(&ind);
        };
    }

    macro_rules! init_w_str {
        ($analyzer:ident, $content:literal, $result:ident) => {
            init!($analyzer);
            let $result = $analyzer.parse_string($content).get_root_entity();
        };
        ($analyzer:ident, $content:literal) => {
            init_w_str!($analyzer, $content, _result);
        }
    }

    #[test]
    fn accepts_simple_c_program() {
        init_w_str!(_analyzer, "int main() {return 0;}", result);
        assert!(result.is_some());
    }

    #[test]
    fn finds_single_ast_node() -> Result<(), String> {
        init_w_str!(analyzer, "int i = 0;");
        let var = analyzer
            .find_first_match(&|e:&Entity| e.get_name().unwrap_or("".to_string()) == "i")
            .ok_or("Not found")?;
        assert_eq!(var.get_kind(), EntityKind::VarDecl);
        assert_eq!(var.get_name().ok_or("No name found")?, "i");
        assert_eq!(var.get_type().ok_or("No type found")?.get_kind(), TypeKind::Int);
        Ok(())
    }

    #[test]
    fn finds_multiple_ast_noes() -> Result<(), String> {
        init_w_str!(analyzer, "int i = 0; float j = 0.0;");
        let vars = analyzer.find_all_matches(
            &|e:&Entity| e.get_kind() == EntityKind::VarDecl);
        assert_eq!(vars.len(), 2);
        assert_eq!(vars.get(0).ok_or("Index error")?
                    .get_kind(), EntityKind::VarDecl);
        assert_eq!(vars.get(0).ok_or("Index error")?
                    .get_name().ok_or("No name found")?
                    , "i");
        assert_eq!(vars.get(0).ok_or("Index error")?
                    .get_type().ok_or("No type found")?
                    .get_kind()
                    , TypeKind::Int);
        assert_eq!(vars.get(1).ok_or("Index error")?
                    .get_kind(), EntityKind::VarDecl);
        assert_eq!(vars.get(1).ok_or("Index error")?
                    .get_name().ok_or("No name found")?
                    , "j");
        assert_eq!(vars.get(1).ok_or("Index error")?
                    .get_type().ok_or("No type found")?
                    .get_kind()
                    , TypeKind::Float);
        Ok(())
    }

    #[test]
    fn finds_func_decl() -> Result<(), String> {
        init_w_str!(analyzer, "int main(){return 0;}");
        let func_decl = analyzer.find_func_def("main").ok_or("Function not found")?;
        assert_eq!(func_decl.get_kind(), EntityKind::FunctionDecl);
        assert!(func_decl.is_definition());
        assert_eq!(func_decl.get_name().ok_or("Name not found")?, "main");
        assert_eq!(func_decl.get_result_type().ok_or("Return type not found")?
                    .get_kind(), TypeKind::Int);
        let args = func_decl.get_arguments().ok_or("Arguments not found")?;
        assert_eq!(args.len(), 0);
        Ok(())
    }

}