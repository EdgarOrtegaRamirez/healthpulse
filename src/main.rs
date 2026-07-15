// Code health and technical debt analyzer
// Scans repositories for complexity, style, and maintainability issues

use clap::Parser;
use healthpulse::analyzer::analyze;
use healthpulse::config::Config;
use healthpulse::output::Output;

#[derive(Parser, Debug)]
#[command(name = "healthpulse")]
#[command(about = "Code health and technical debt analyzer", long_about = None)]
struct Cli {
    /// Path to scan (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    path: String,

    /// Output format: text, json, markdown
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Maximum allowed cyclomatic complexity
    #[arg(short, long, default_value = "10")]
    max_complexity: u32,

    /// Maximum function length (lines)
    #[arg(short, long, default_value = "50")]
    max_function_length: u32,

    /// Maximum file length (lines)
    #[arg(long, default_value = "500")]
    max_file_length: u32,

    /// Minimum test file ratio (0.0 - 1.0)
    #[arg(long, default_value = "0.3")]
    min_test_ratio: f64,

    /// Config file path
    #[arg(short, long)]
    config: Option<String>,

    /// Ignore patterns (can be specified multiple times)
    #[arg(long)]
    ignore: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    let mut config = Config::default();
    config.max_complexity = cli.max_complexity;
    config.max_function_length = cli.max_function_length;
    config.max_file_length = cli.max_file_length;
    config.min_test_ratio = cli.min_test_ratio;

    if let Some(cfg_path) = &cli.config {
        if let Err(e) = config.load(cfg_path) {
            eprintln!("Error loading config: {e}");
            std::process::exit(1);
        }
    }

    for pattern in &cli.ignore {
        config.add_ignore(pattern);
    }

    let output = match cli.format.as_str() {
        "json" => Output::Json,
        "markdown" => Output::Markdown,
        _ => Output::Text,
    };

    let result = analyze(&cli.path, &config);

    println!("{}", output.render(&result, &config));

    // Exit with non-zero if there are critical issues
    if result.critical > 0 {
        std::process::exit(2);
    }
}