use structopt::clap::Shell::Bash;

include!("src/cli/mod.rs");

fn main() {
    let mut app = Args::clap();
    std::fs::create_dir_all("./target/").unwrap();
    app.gen_completions("pantsu-tags", Bash, "./target/");
}