use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Process a specific slot number and exit (if not provided, will continuously watch for new slots)
    #[arg(short, long)]
    slot: Option<u64>,

    /// Dry run mode - only show JSON without calling trigger API
    #[arg(short, long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    chase::run(args.slot, args.dry_run).await;
}
