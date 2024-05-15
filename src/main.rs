use clap::Parser;

#[derive(Parser, Debug)]
#[command(version = "0.1", about = "A simple backup tool" , long_about = None)]
struct Args {
    /// The name of the person to greet
    #[arg(short, long)]
    name: String,

    /// The welcome message
    #[arg(short, long, default_value_t = String::from("Welcome to flux"))]
    welcome: String,
}

fn main() {
    let args = Args::parse();
    println!("Hello {}", args.name);
    println!("{}", args.welcome);
}
