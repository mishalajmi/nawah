use clap::Parser;
use nawah_core::Context;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "nawah")]
#[command(about = "A self-host PaaS to Netlify and other platforms", long_about = None)]
struct Args {
    app_path: PathBuf,

    #[arg(short, long)]
    build: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let context = Context::default();

    context.load_application(&args.app_path, args.build).await?;

    Ok(())
}
