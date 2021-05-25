extern crate libpartlibspec;
use clap::{AppSettings, Clap};
use libpartlibspec::predinfer::predgen;
use std::fs;
use rand::Rng;

#[derive(Clap)]
#[clap(version = "0.1.0")]
#[clap(setting = AppSettings::ColoredHelp)]
struct PGConfig {
    #[clap(about="Grammar file to read grammar from")]
    grammar: String,
    #[clap(short, long, about="Suppress stderr messages")]
    quiet: bool,
    #[clap(subcommand)]
    subcmd: PGSubCmd,
}

#[derive(Clap)]
enum PGSubCmd {
    #[clap(about="Randomly generate ASTs")]
    Random(PGRandom)
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct PGRandom {
    #[clap(short, long, about="Max height of AST to be generated")]
    height: usize,
    #[clap(short, long, about="Min height of AST to be generated", default_value="1")]
    min_height: usize,
    #[clap(short, long, about="Number of AST to be generated")]
    number: usize,
    #[clap(short, long, about="Generate all ASTs at max height")]
    fixed: bool,
    #[clap(short, long, about="Retry after failure, until number of ASTs are successfully generated")]
    retry: bool

}

fn generate_random_ast(gen: &predgen::PredGenerator, random_config: &PGRandom) -> Result<predgen::Node, usize> {
    let height = if random_config.fixed {
        random_config.height
    } else {
        let mut rng = rand::thread_rng();
        rng.gen_range(random_config.min_height, random_config.height+1)
    };
    gen.generate_random_full_ast(height).ok_or(height)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PGConfig::parse();
    let grammar_content = fs::read_to_string(&config.grammar)?;
    let grammar = predgen::Grammar::from_input(serde_yaml::from_str(grammar_content.as_str())?);
    let gen = predgen::PredGenerator::new(&grammar);
    match config.subcmd {
        PGSubCmd::Random(ref random_config) => {
            let mut i = 0;
            while i < random_config.number {
                match generate_random_ast(&gen, random_config) {
                    Ok(ast) => {
                        if !config.quiet {
                            eprintln!("Height {}:", ast.get_height());
                        }
                        println!("{}", ast.to_string());
                    },
                    Err(error_height) => {
                        if !config.quiet {
                            eprintln!("Generation Failure at height {}", error_height);
                        }
                        if random_config.retry {
                            continue;
                        }
                    }
                }
                i += 1;
            }
        }
    };
    Ok(())
}