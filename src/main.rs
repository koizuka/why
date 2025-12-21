use clap::Parser;
use colored::Colorize;
use why::{detect_command, Cli, Confidence, DetectionResult, OutputFormat};

fn main() {
    let cli = Cli::parse();

    // Handle --json shortcut
    let format = if cli.json {
        OutputFormat::Json
    } else {
        cli.format
    };

    match detect_command(&cli.command, cli.verbose) {
        Ok(result) => {
            print_result(&result, format);
        }
        Err(e) => {
            eprintln!("{}: {}", "Error".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn print_result(result: &DetectionResult, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result).unwrap());
        }
        OutputFormat::Short => {
            println!("{}", result.manager_id);
        }
        OutputFormat::Text => {
            print_text_result(result);
        }
    }
}

fn print_text_result(result: &DetectionResult) {
    let confidence_str = match result.confidence {
        Confidence::High => "(verified)".green(),
        Confidence::Medium => "(likely)".yellow(),
        Confidence::Low => "(possible)".yellow(),
        Confidence::Uncertain => "(uncertain)".red(),
    };

    println!(
        "{} was installed by: {} {}",
        result
            .command_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .bold(),
        result.manager_name.cyan().bold(),
        confidence_str
    );

    if let Some(ref package) = result.package_name {
        println!("  {}: {}", "Package".dimmed(), package);
    }

    if let Some(ref version) = result.version {
        println!("  {}: {}", "Version".dimmed(), version);
    }

    println!(
        "  {}: {}",
        "Location".dimmed(),
        result.resolved_path.display()
    );
}
