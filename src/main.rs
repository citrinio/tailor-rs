use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tailor::cmds::{build::cmd_build, new::cmd_new};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    New {
        path: PathBuf,
        #[clap(long, action, help = "Use a binary (application) template [default]")]
        bin: bool,
        #[clap(long, action, help = "Use a library template")]
        lib: bool,
    },
    Build {
        #[clap(
            short,
            long,
            action,
            help = "Build in release mode, with optimizations"
        )]
        release: bool,
        #[clap(short, long, help = "Number of parallel jobs, defaults to # of CPUs.")]
        jobs: Option<u8>,
    },
    Clean {
        #[clap(
            short,
            long,
            action,
            help = "Build in release mode, with optimizations"
        )]
        release: bool,
    },
    Run {
        #[clap(
            short,
            long,
            action,
            help = "Build in release mode, with optimizations"
        )]
        release: bool,
        #[clap(short, long, help = "Number of parallel jobs, defaults to # of CPUs.")]
        jobs: Option<u8>,
    },
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let default_jobs = num_cpus::get();

    match cli.command {
        Command::New { path, bin, lib } => cmd_new(path, bin, lib),
        Command::Build { release, jobs } => cmd_build(release, jobs.unwrap_or(default_jobs as u8)),
        Command::Clean { release } => todo!(),
        Command::Run { release, jobs } => todo!(),
    }
}
