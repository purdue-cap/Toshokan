#[allow(unused_imports)]
use pest::Parser;

#[derive(Parser)]
#[grammar = "frontend/sketch.pest"]
pub struct SketchParser;

#[cfg(test)]
mod tests {
    use super::*;

    fn accepts_string(content: &str) -> bool {
        let result = SketchParser::parse(Rule::program, &content);
        result.is_ok()
    }

    #[test]
    fn accepts_empty_input() {
        assert_eq!(accepts_string(""), true);
    }

    #[test]
    fn accepts_simple_sketch() {
        let simple_sketch = r#"
        void main() {
            int i = 0;
        }
        "#;
        assert_eq!(accepts_string(simple_sketch), true);
    }

    #[test]
    fn rejects_sketch_without_semicolon() {
        let sketch_without_semicolon = r#"
        void main() {
            int i = 0
        }
        "#;
        assert_eq!(accepts_string(sketch_without_semicolon), false);
    }

    #[test]
    fn accepts_simple_sketch_with_hole() {
        let simple_sketch_with_hole = r#"
        harness void main(int i) {
            assume i > 0;
            int j = i + ??;
            assert j > 1;
        }
        "#;
        assert_eq!(accepts_string(simple_sketch_with_hole), true);
    }

}