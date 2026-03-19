use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::process;

use png_parser::commands::mode;

#[derive(Parser)]
#[command(
    name    = "png-parser-cli",
    author  = "Pranjal Panging",
    version = "0.3.0",
    about   = "Hide any file inside PNG/BMP/TIFF/WebP — compress, encrypt, embed.",
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Hide(HideArgs),
    Reveal(RevealArgs),
    Info(InfoArgs),
    Verify(VerifyArgs),
    Delete(DeleteArgs),
    Reencrypt(ReencryptArgs),
    Capacity(CapacityArgs),
    Fingerprint(FingerprintArgs),
    Inspect(InspectArgs),
    Strip(StripArgs),
    Split(SplitArgs),
    Merge(MergeArgs),
}

#[derive(Args)]
struct ExpiryArgs {
    #[arg(long, value_name = "N")]
    days: Option<i64>,
    #[arg(long, value_name = "N")]
    hours: Option<i64>,
    #[arg(long, value_name = "N")]
    minutes: Option<i64>,
    #[arg(long, value_name = "N")]
    seconds: Option<i64>,
}

impl ExpiryArgs {
    fn has_expiry(&self) -> bool {
        self.days.is_some()
            || self.hours.is_some()
            || self.minutes.is_some()
            || self.seconds.is_some()
    }

    fn to_tuple(&self) -> Option<(Option<i64>, Option<i64>, Option<i64>, Option<i64>)> {
        if self.has_expiry() {
            Some((self.days, self.hours, self.minutes, self.seconds))
        } else {
            None
        }
    }
}

#[derive(Args)]
struct HideArgs {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    file: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
    #[arg(short, long)]
    password: Option<String>,
    #[arg(long, default_value = "chunk")]
    mode: String,
    #[command(flatten)]
    expiry: ExpiryArgs,
}

#[derive(Args)]
struct RevealArgs {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
    #[arg(short, long)]
    password: Option<String>,
}

#[derive(Args)]
struct InfoArgs {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    password: Option<String>,
}

#[derive(Args)]
struct VerifyArgs {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    password: String,
}

#[derive(Args)]
struct DeleteArgs {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
    #[arg(short, long)]
    password: Option<String>,
}

#[derive(Args)]
struct ReencryptArgs {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
    #[arg(long)]
    old_password: String,
    #[arg(long)]
    new_password: String,
}

#[derive(Args)]
struct CapacityArgs {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(long, default_value = "chunk")]
    mode: String,
}

#[derive(Args)]
struct FingerprintArgs {
    #[arg(short, long)]
    input: PathBuf,
}

#[derive(Args)]
struct InspectArgs {
    #[arg(short, long)]
    input: PathBuf,
}

#[derive(Args)]
struct StripArgs {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
}

#[derive(Args)]
struct SplitArgs {
    #[arg(short, long)]
    file: PathBuf,
    #[arg(short, long, num_args = 1..)]
    carriers: Vec<PathBuf>,
    #[arg(short, long)]
    output_dir: PathBuf,
    #[arg(short, long)]
    password: Option<String>,
    #[command(flatten)]
    expiry: ExpiryArgs,
}

#[derive(Args)]
struct MergeArgs {
    #[arg(short, long, num_args = 1..)]
    inputs: Vec<PathBuf>,
    #[arg(short, long)]
    output: PathBuf,
    #[arg(short, long)]
    password: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Hide(a)        => run_hide(a),
        Commands::Reveal(a)      => run_reveal(a),
        Commands::Info(a)        => run_info(a),
        Commands::Verify(a)      => run_verify(a),
        Commands::Delete(a)      => run_delete(a),
        Commands::Reencrypt(a)   => run_reencrypt(a),
        Commands::Capacity(a)    => run_capacity(a),
        Commands::Fingerprint(a) => run_fingerprint(a),
        Commands::Inspect(a)     => run_inspect(a),
        Commands::Strip(a)       => run_strip(a),
        Commands::Split(a)       => run_split(a),
        Commands::Merge(a)       => run_merge(a),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run_hide(args: HideArgs) -> Result<(), String> {
    mode::hide(
        p(&args.input),
        p(&args.file),
        p(&args.output),
        args.password.as_deref(),
        &args.mode,
        args.expiry.to_tuple(),
    ).map_err(|e| e.to_string())?;
    println!("Hidden '{}' in '{}' → '{}'", p(&args.file), p(&args.input), p(&args.output));
    Ok(())
}

fn run_reveal(args: RevealArgs) -> Result<(), String> {
    let info = mode::reveal(
        p(&args.input),
        p(&args.output),
        args.password.as_deref(),
    ).map_err(|e| e.to_string())?;
    println!("Revealed: {}", info);
    Ok(())
}

fn run_info(args: InfoArgs) -> Result<(), String> {
    let info = mode::info(
        p(&args.input),
        args.password.as_deref(),
    ).map_err(|e| e.to_string())?;
    println!("{}", info);
    Ok(())
}

fn run_verify(args: VerifyArgs) -> Result<(), String> {
    let ok = mode::verify(
        p(&args.input),
        &args.password,
    ).map_err(|e| e.to_string())?;
    if ok {
        println!("Password correct.");
    } else {
        println!("Wrong password.");
        process::exit(1);
    }
    Ok(())
}

fn run_delete(args: DeleteArgs) -> Result<(), String> {
    mode::delete(
        p(&args.input),
        p(&args.output),
        args.password.as_deref(),
    ).map_err(|e| e.to_string())?;
    println!("Payload removed from '{}' → '{}'", p(&args.input), p(&args.output));
    Ok(())
}

fn run_reencrypt(args: ReencryptArgs) -> Result<(), String> {
    mode::reencrypt(
        p(&args.input),
        p(&args.output),
        &args.old_password,
        &args.new_password,
    ).map_err(|e| e.to_string())?;
    println!("Re-encrypted '{}' → '{}'", p(&args.input), p(&args.output));
    Ok(())
}

fn run_capacity(args: CapacityArgs) -> Result<(), String> {
    let bytes = mode::capacity(
        p(&args.input),
        &args.mode,
    ).map_err(|e| e.to_string())?;
    println!("{} can hold {} bytes ({} KB) in {} mode",
        p(&args.input), bytes, bytes / 1024, args.mode);
    Ok(())
}

fn run_fingerprint(args: FingerprintArgs) -> Result<(), String> {
    let fp = mode::fingerprint(p(&args.input))
        .map_err(|e| e.to_string())?;
    println!("Fingerprint: {}", fp);
    Ok(())
}

fn run_inspect(args: InspectArgs) -> Result<(), String> {
    mode::inspect(p(&args.input))
        .map_err(|e| e.to_string())
}

fn run_strip(args: StripArgs) -> Result<(), String> {
    mode::strip(p(&args.input), p(&args.output))
        .map_err(|e| e.to_string())?;
    println!("Metadata stripped '{}' → '{}'", p(&args.input), p(&args.output));
    Ok(())
}

fn run_split(args: SplitArgs) -> Result<(), String> {
    let carriers: Vec<&str> = args.carriers
        .iter()
        .map(|p| p.to_str().unwrap_or(""))
        .collect();

    let outputs = mode::split(
        p(&args.file),
        &carriers,
        p(&args.output_dir),
        args.password.as_deref(),
        args.expiry.to_tuple(),
    ).map_err(|e| e.to_string())?;

    println!("Split into {} shards:", outputs.len());
    for o in &outputs {
        println!("  {}", o);
    }
    Ok(())
}

fn run_merge(args: MergeArgs) -> Result<(), String> {
    let inputs: Vec<&str> = args.inputs
        .iter()
        .map(|p| p.to_str().unwrap_or(""))
        .collect();

    let output = mode::merge(
        &inputs,
        p(&args.output),
        args.password.as_deref(),
    ).map_err(|e| e.to_string())?;

    println!("Merged → '{}'", output);
    Ok(())
}

fn p(pb: &PathBuf) -> &str {
    pb.to_str().unwrap_or_else(|| {
        eprintln!("Error: path contains non-UTF-8 characters");
        process::exit(1);
    })
}