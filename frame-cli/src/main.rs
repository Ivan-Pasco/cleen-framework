use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

mod commands;
mod compiler;
mod manifest;
mod openapi;
mod sdk_typescript;

#[derive(Parser)]
#[command(name = "frame")]
#[command(author = "Ivan Pasco")]
#[command(version = "0.1.0")]
#[command(about = "Frame - The official full-stack framework for Clean Language", long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Create a new Frame project
	New {
		/// Name of the project
		name: String,

		/// Template to use (default: full-stack)
		#[arg(short, long, default_value = "full-stack")]
		template: String,
	},

	/// Start development server with hot reload
	Serve {
		/// Port to listen on
		#[arg(short, long, default_value = "3000")]
		port: u16,

		/// Host to bind to
		#[arg(long, default_value = "127.0.0.1")]
		host: String,
	},

	/// Build the project for production
	Build {
		/// Build target (web, pwa, mobile, desktop, server, cli)
		#[arg(short, long, default_value = "web")]
		target: String,

		/// Enable optimizations
		#[arg(short, long)]
		release: bool,
	},

	/// Database commands
	Db {
		#[command(subcommand)]
		action: DbCommands,
	},

	/// API commands
	Api {
		#[command(subcommand)]
		action: ApiCommands,
	},

	/// Platform-specific initialization
	#[command(subcommand)]
	Platform(PlatformCommands),
}

#[derive(Subcommand)]
enum DbCommands {
	/// Create a new migration file
	Create {
		/// Migration name (e.g., create_users_table)
		name: String,
	},

	/// Apply pending migrations
	Migrate {
		/// Number of migrations to apply (default: all)
		#[arg(short, long)]
		steps: Option<usize>,
	},

	/// Rollback migrations
	Rollback {
		/// Number of migrations to rollback (default: 1)
		#[arg(short, long, default_value = "1")]
		steps: usize,
	},

	/// Show migration status
	Status,

	/// Execute database seeding script
	Seed {
		/// Path to seed file
		#[arg(short, long, default_value = "db/seed.cln")]
		file: PathBuf,
	},
}

#[derive(Subcommand)]
enum ApiCommands {
	/// Generate OpenAPI 3.1 specification
	Spec {
		/// Output file
		#[arg(short, long, default_value = "api/openapi.json")]
		output: PathBuf,
	},

	/// Generate client SDKs
	Sdk {
		/// Target language (clean, typescript, swift, kotlin)
		#[arg(short, long)]
		lang: String,

		/// Output directory
		#[arg(short, long)]
		output: PathBuf,
	},
}

#[derive(Subcommand)]
enum PlatformCommands {
	/// Initialize PWA wrapper
	#[command(name = "pwa:init")]
	PwaInit,

	/// Initialize mobile wrapper (Capacitor)
	#[command(name = "mobile:init")]
	MobileInit,

	/// Add mobile plugin
	#[command(name = "mobile:plugin")]
	MobilePlugin {
		/// Plugin name (camera, filesystem, push, etc.)
		name: String,
	},

	/// Initialize desktop wrapper (Tauri)
	#[command(name = "desktop:init")]
	DesktopInit,

	/// Add desktop adapter
	#[command(name = "desktop:adapter")]
	DesktopAdapter {
		/// Adapter name (fs, dialog, etc.)
		name: String,
	},

	/// Initialize server deployment files
	#[command(name = "server:init")]
	ServerInit,
}

#[tokio::main]
async fn main() -> Result<()> {
	// Initialize logging
	tracing_subscriber::fmt::init();

	let cli = Cli::parse();

	match cli.command {
		Commands::New { name, template } => {
			commands::new::create_project(&name, &template).await?;
		}
		Commands::Serve { port, host } => {
			commands::serve::start_dev_server(&host, port).await?;
		}
		Commands::Build { target, release } => {
			commands::build::build_project(&target, release).await?;
		}
		Commands::Db { action } => match action {
			DbCommands::Create { name } => {
				commands::db::create_migration(&name).await?;
			}
			DbCommands::Migrate { steps } => {
				commands::db::migrate_up(steps).await?;
			}
			DbCommands::Rollback { steps } => {
				commands::db::migrate_down(steps).await?;
			}
			DbCommands::Status => {
				commands::db::migration_status().await?;
			}
			DbCommands::Seed { file } => {
				commands::db::seed_database(&file).await?;
			}
		},
		Commands::Api { action } => match action {
			ApiCommands::Spec { output } => {
				commands::api::generate_openapi_spec(&output).await?;
			}
			ApiCommands::Sdk { lang, output } => {
				commands::api::generate_sdk(&lang, &output).await?;
			}
		},
		Commands::Platform(platform) => match platform {
			PlatformCommands::PwaInit => {
				commands::platform::init_pwa().await?;
			}
			PlatformCommands::MobileInit => {
				commands::platform::init_mobile().await?;
			}
			PlatformCommands::MobilePlugin { name } => {
				commands::platform::add_mobile_plugin(&name).await?;
			}
			PlatformCommands::DesktopInit => {
				commands::platform::init_desktop().await?;
			}
			PlatformCommands::DesktopAdapter { name } => {
				commands::platform::add_desktop_adapter(&name).await?;
			}
			PlatformCommands::ServerInit => {
				commands::platform::init_server().await?;
			}
		},
	}

	Ok(())
}
