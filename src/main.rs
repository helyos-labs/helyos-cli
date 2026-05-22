mod client;
mod commands;
mod output;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "nexa",
    about = "NexaNet CLI — deploy and manage containers",
    version,
    propagate_version = true
)]
struct Cli {
    #[arg(long, default_value = "http://localhost:6443", global = true)]
    server: String,

    /// Output results as JSON
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new NexaNet project
    Init {
        /// Project name (interactive if omitted)
        name: Option<String>,
        /// Container image to use
        #[arg(long)]
        image: Option<String>,
    },

    /// Deploy a service from a YAML spec
    Deploy {
        /// Path to the deployment YAML file
        file: String,
    },

    /// Show cluster status overview
    Status,

    /// List all pods
    Pods {
        /// Filter by project
        #[arg(short, long)]
        project: Option<String>,
    },

    /// List all deployments
    Deployments {
        /// Filter by project
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Stream logs from a deployment
    Logs {
        /// Deployment name
        name: String,

        /// Project name
        #[arg(short, long)]
        project: Option<String>,

        /// Number of lines to tail
        #[arg(long)]
        tail: Option<u64>,
    },

    /// Scale a deployment
    Scale {
        /// Deployment name
        name: String,

        /// Number of replicas
        replicas: u32,

        /// Project name
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Stop a deployment
    Stop {
        /// Deployment name
        name: String,

        /// Project name
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Remove a deployment
    Rm {
        /// Deployment name
        name: String,

        /// Project name
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Manage projects
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },

    /// Manage secrets
    Secret {
        #[command(subcommand)]
        command: SecretCommands,
    },

    /// List cluster nodes
    Nodes,

    /// Manage a specific node
    Node {
        #[command(subcommand)]
        command: NodeCommands,
    },

    /// Manage the cluster
    Cluster {
        #[command(subcommand)]
        command: ClusterCommands,
    },
}

#[derive(Subcommand)]
enum ProjectCommands {
    /// List all projects
    List,

    /// Create a new project
    Create {
        /// Project name
        name: String,
    },

    /// Suspend a project (stops all deployments)
    Suspend {
        /// Project name
        name: String,
    },

    /// Resume a suspended project
    Resume {
        /// Project name
        name: String,
    },

    /// Delete a project and all its resources
    Delete {
        /// Project name
        name: String,
    },
}

#[derive(Subcommand)]
enum SecretCommands {
    /// Set a secret value
    Set {
        /// Secret name
        name: String,
        /// Secret value
        value: String,
        /// Project name
        #[arg(short, long)]
        project: String,
    },

    /// List all secrets in a project
    List {
        /// Project name
        #[arg(short, long)]
        project: String,
    },

    /// Remove a secret
    Rm {
        /// Secret name
        name: String,
        /// Project name
        #[arg(short, long)]
        project: String,
    },
}

#[derive(Subcommand)]
enum ClusterCommands {
    /// Initialize the cluster and generate a join token
    Init,
    /// Manage join tokens
    Token {
        #[command(subcommand)]
        command: TokenCommands,
    },
    /// View or update scheduler configuration
    Config {
        #[command(subcommand)]
        command: ClusterConfigCommands,
    },
}

#[derive(Subcommand)]
enum ClusterConfigCommands {
    /// Get current scheduler configuration
    GetScheduler,
    /// Set scheduler strategy or individual weight
    Set {
        /// Key: "scheduler" for strategy, or "scheduler.weights.<name>" for individual weight
        key: String,
        /// Value: strategy name (spread/binpack) or weight number
        value: String,
    },
}

#[derive(Subcommand)]
enum TokenCommands {
    /// Show the current join token
    Show,
    /// Rotate the join token
    Rotate,
}

#[derive(Subcommand)]
enum NodeCommands {
    /// Drain a node (stop scheduling new pods)
    Drain {
        /// Node name
        name: String,
    },
    /// Remove a node from the cluster
    Rm {
        /// Node name
        name: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    output::set_json_mode(cli.json);
    let client = client::NexaClient::new(&cli.server);

    let result = match cli.command {
        Commands::Init { name, image } => {
            commands::init(name.as_deref(), image.as_deref())
        }
        Commands::Deploy { file } => commands::deploy(&client, &file).await,
        Commands::Status => commands::status(&client).await,
        Commands::Pods { project } => commands::pods(&client, project.as_deref()).await,
        Commands::Deployments { project } => {
            commands::deployments(&client, project.as_deref()).await
        }
        Commands::Logs {
            name,
            project,
            tail,
        } => commands::logs(&client, project.as_deref(), &name, tail).await,
        Commands::Scale {
            name,
            replicas,
            project,
        } => commands::scale(&client, project.as_deref(), &name, replicas).await,
        Commands::Stop { name, project } => {
            commands::stop(&client, project.as_deref(), &name).await
        }
        Commands::Rm { name, project } => {
            commands::remove(&client, project.as_deref(), &name).await
        }
        Commands::Project { command } => match command {
            ProjectCommands::List => commands::list_projects(&client).await,
            ProjectCommands::Create { name } => commands::create_project(&client, &name).await,
            ProjectCommands::Suspend { name } => commands::suspend_project(&client, &name).await,
            ProjectCommands::Resume { name } => commands::resume_project(&client, &name).await,
            ProjectCommands::Delete { name } => commands::delete_project(&client, &name).await,
        },
        Commands::Secret { command } => match command {
            SecretCommands::Set { name, value, project } => {
                commands::secret::set(&client, &project, &name, &value).await
            }
            SecretCommands::List { project } => {
                commands::secret::list(&client, &project).await
            }
            SecretCommands::Rm { name, project } => {
                commands::secret::remove(&client, &project, &name).await
            }
        },
        Commands::Nodes => commands::nodes(&client).await,
        Commands::Cluster { command } => match command {
            ClusterCommands::Init => commands::cluster::init(&client).await,
            ClusterCommands::Token { command } => match command {
                TokenCommands::Show => commands::cluster::token_show(&client).await,
                TokenCommands::Rotate => commands::cluster::token_rotate(&client).await,
            },
            ClusterCommands::Config { command } => match command {
                ClusterConfigCommands::GetScheduler => {
                    commands::cluster::get_scheduler_config(&client).await
                }
                ClusterConfigCommands::Set { key, value } => {
                    commands::cluster::set_cluster_config(&client, &key, &value).await
                }
            },
        },
        Commands::Node { command } => match command {
            NodeCommands::Drain { name } => commands::node::drain(&client, &name).await,
            NodeCommands::Rm { name } => commands::node::remove(&client, &name).await,
        },
    };

    if let Err(e) = result {
        let msg = e.to_string();
        if msg.contains("Connection refused") || msg.contains("connect") {
            output::print_error_with_hint(
                "Cannot connect to nexad",
                &format!("Is nexad running? Start it with: nexad --host {}", cli.server),
            );
        } else {
            output::print_error(&msg);
        }
        std::process::exit(1);
    }

    Ok(())
}
