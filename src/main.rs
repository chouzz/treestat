use treestat::cli::{Cli, print_help};

fn main() {
    match Cli::parse_env() {
        Ok(cli) => match treestat::run(cli) {
            Ok(output) => {
                print!("{output}");
            }
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        },
        Err(msg) => {
            if msg == "--help" {
                print_help();
                return;
            }
            if msg == "--version" {
                println!("treestat 0.1.0");
                return;
            }
            eprintln!("error: {msg}");
            eprintln!("use --help for usage.");
            std::process::exit(2);
        }
    }
}
