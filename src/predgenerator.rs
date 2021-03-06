use clap::Parser;
use libpartlibspec::predinfer::predgen;
use std::fs;
use rand::Rng;

#[derive(Parser)]
#[clap(version = "0.1.0", about)]
struct PGConfig {
    /// Grammar file to read grammar from
    grammar: String,
    #[clap(short, long)]
    /// Suppress stderr messages
    quiet: bool,
    #[clap(subcommand)]
    subcmd: PGSubCmd,
}

#[derive(Parser)]
#[clap(about)]
enum PGSubCmd {
    /// Randomly generate ASTs
    Random(PGRandom),
    /// Generate all ASTs based on height
    Genallh(PGGenallh),
    /// Generate all ASTs based on size
    Genall(PGGenall)
}

#[derive(Parser)]
#[clap(about)]
struct PGRandom {
    #[clap(short, long)]
    /// Max height of AST to be generated
    height: usize,
    #[clap(short, long, default_value="1")]
    /// Min height of AST to be generated
    min_height: usize,
    #[clap(short, long)]
    /// Number of AST to be generated
    number: usize,
    #[clap(short, long)]
    /// Generate all ASTs at max height
    fixed: bool,
    #[clap(short, long)]
    /// Retry after failure, until number of ASTs are successfully generated
    retry: bool

}

#[derive(Parser)]
#[clap(about)]
struct PGGenallh {
    #[clap(short, long)]
    /// Max height of AST to be generated
    max_height: usize
}

#[derive(Parser)]
#[clap(about)]
struct PGGenall {
    #[clap(short, long)]
    /// Size of AST to be generated
    size: usize
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

fn generate_all_ast_with_height(gen: &mut predgen::PredGenerator, genall_config: &PGGenallh) -> Vec<predgen::Node> { 
    gen.generate_all_ast_with_height(genall_config.max_height)
}

fn generate_all_ast_with_size(gen: &mut predgen::PredGenerator, genall_config: &PGGenall) -> Vec<predgen::Node> {
    gen.generate_all_ast_with_size(genall_config.size)
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PGConfig::parse();
    let grammar_content = fs::read_to_string(&config.grammar)?;
    let grammar = predgen::Grammar::from_input(serde_yaml::from_str(grammar_content.as_str())?);
    let mut gen = predgen::PredGenerator::new(&grammar);
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
        PGSubCmd::Genallh(ref genall_config) => {
            let ast = generate_all_ast_with_height(&mut gen, genall_config);
            let ast_p = ast.into_iter().map(|node| node.to_string()).collect::<Vec<_>>();
            for i in &ast_p{
                println!("{}",i);  
            }
                 
        }
        PGSubCmd::Genall(ref genall_config) => {
            let ast = generate_all_ast_with_size(&mut gen, genall_config);
            let ast_p = ast.into_iter().map(|node| node.to_string()).collect::<Vec<_>>();
            for i in &ast_p{
                println!("{}",i);  
            }
                 
        }
    };
    Ok(())
}