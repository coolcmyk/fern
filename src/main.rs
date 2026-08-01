use std::env;

use clap::{Args, Parser, Subcommand, ValueEnum};
use fern::{
    Config, Error, Result, profile,
    provider::{
        ComputeKind, Provider,
        runpod::{CreatePodRequest, RunpodClient},
    },
};

#[derive(Debug, Parser)]
#[command(name = "fern", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate Fern's local configuration without printing secrets.
    Config(ConfigArgs),
    /// Create a workload from a built-in deployment profile.
    Deploy(DeployArgs),
    /// Inspect Runpod Pods.
    Pod(PodArgs),
}

#[derive(Debug, Args)]
struct DeployArgs {
    /// Built-in workload profile.
    #[arg(long, value_enum, default_value = "drone-sim-stack")]
    profile: ProfileArg,

    /// Override the profile's container image.
    #[arg(long)]
    image: Option<String>,

    /// Runpod container registry credential ID for a private image.
    #[arg(long)]
    registry_auth_id: Option<String>,

    /// Smoke-test duration in seconds.
    #[arg(long, default_value_t = 300)]
    duration: u32,

    /// Print the Runpod request without creating a Pod.
    #[arg(long, conflicts_with = "yes")]
    dry_run: bool,

    /// Confirm creation of a billable Runpod Pod.
    #[arg(long, conflicts_with = "dry_run")]
    yes: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ProfileArg {
    #[value(alias = "drone-sim-lane-a")]
    DroneSimStack,
}

#[derive(Debug, Args)]
struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    /// Check credential resolution and the Runpod API endpoint.
    Check,
}

#[derive(Debug, Args)]
struct PodArgs {
    #[command(subcommand)]
    command: PodCommand,
}

#[derive(Debug, Subcommand)]
enum PodCommand {
    /// List Pods visible to the configured account.
    List {
        /// Restrict results to CPU or GPU Pods.
        #[arg(long, value_enum)]
        compute: Option<ComputeArg>,
    },
    /// Get one Pod by provider ID.
    Get {
        /// Runpod Pod ID.
        id: String,
    },
    /// Stop a running Pod while retaining its persistent volume.
    Stop {
        /// Runpod Pod ID.
        id: String,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ComputeArg {
    Cpu,
    Gpu,
}

impl From<ComputeArg> for ComputeKind {
    fn from(value: ComputeArg) -> Self {
        match value {
            ComputeArg::Cpu => Self::Cpu,
            ComputeArg::Gpu => Self::Gpu,
        }
    }
}

#[tokio::main]
async fn main() {
    // Existing process variables retain precedence over project-local values.
    // Missing .env files are valid for installed binaries.
    let _ = dotenvy::dotenv();

    if let Err(error) = run(Cli::parse()).await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Config(args) => match args.command {
            ConfigCommand::Check => {
                let config = Config::from_env()?;
                print_json(&serde_json::json!({
                    "ok": true,
                    "provider": "runpod",
                    "api_base": config.runpod_api_base().as_str(),
                    "credential_source": config.credential_source(),
                }));
            }
        },
        Command::Deploy(args) => {
            if args.duration == 0 {
                return Err(Error::Config("--duration must be at least 1 second".into()));
            }

            let registry_auth_id = args
                .registry_auth_id
                .or_else(|| optional_env("RUNPOD_CONTAINER_REGISTRY_AUTH_ID"));
            let spec = match args.profile {
                ProfileArg::DroneSimStack => {
                    profile::drone_sim_stack(args.image, args.duration, registry_auth_id)
                }
            };

            if args.dry_run {
                print_json(&CreatePodRequest::from(spec));
                return Ok(());
            }

            if !args.yes {
                return Err(Error::Config(
                    "deploy creates a billable Pod; inspect with --dry-run or confirm with --yes"
                        .into(),
                ));
            }

            let config = Config::from_env()?;
            let provider =
                RunpodClient::new(config.runpod_api_base().clone(), config.runpod_api_key())?;
            let pod = provider.create(spec).await?;
            print_json(&pod);
        }
        Command::Pod(args) => {
            let config = Config::from_env()?;
            let provider =
                RunpodClient::new(config.runpod_api_base().clone(), config.runpod_api_key())?;

            match args.command {
                PodCommand::List { compute } => {
                    let pods = provider.list(compute.map(ComputeKind::from)).await?;
                    print_json(&pods);
                }
                PodCommand::Get { id } => {
                    let pod = provider.get(&id).await?;
                    print_json(&pod);
                }
                PodCommand::Stop { id } => {
                    let pod = provider.stop(&id).await?;
                    print_json(&pod);
                }
            }
        }
    }

    Ok(())
}

fn optional_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn print_json(value: &impl serde::Serialize) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).expect("Fern output must serialize")
    );
}
