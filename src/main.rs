mod client;
mod commands;
mod config;
mod output;
mod tui;

use std::io::IsTerminal;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    name = "helyos",
    about = "Helyos CLI — deploy and manage containers",
    version,
    propagate_version = true
)]
struct Cli {
    /// helyosd server URL (or set HELYOS_SERVER, or `server` in ~/.helyos/config.toml;
    /// defaults to http://localhost:6443)
    #[arg(long, env = "HELYOS_SERVER", global = true)]
    server: Option<String>,

    /// API bearer token (or set HELYOS_API_TOKEN env var, or `token` in ~/.helyos/config.toml)
    #[arg(long, env = "HELYOS_API_TOKEN", global = true)]
    token: Option<String>,

    /// Output results as JSON
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Helyos project
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

        /// Timeout in seconds waiting for pods to become ready
        #[arg(long, default_value = "60")]
        timeout: u64,
    },

    /// Show cluster status overview
    Status,

    /// Live cluster dashboard
    Top,

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

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
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

    /// List all routes
    Routes {
        /// Filter by project
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Manage routes
    Route {
        #[command(subcommand)]
        command: RouteCommands,
    },

    /// Manage TLS certificates
    Cert {
        #[command(subcommand)]
        command: CertCommands,
    },

    /// Setup system components
    Setup {
        #[command(subcommand)]
        component: SetupComponent,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
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

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum SecretCommands {
    /// Set a secret value
    Set {
        /// Secret name
        name: String,
        /// Secret value (if omitted, reads from stdin or prompts interactively)
        #[arg(long)]
        value: Option<String>,
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

#[derive(Subcommand)]
enum RouteCommands {
    /// Add a route for a domain
    Add {
        /// Domain name
        domain: String,
        /// Project name
        #[arg(short, long)]
        project: String,
        /// Deployment name
        #[arg(long)]
        deployment: String,
        /// Enable automatic HTTPS
        #[arg(long)]
        https: bool,
    },
    /// Remove a route
    Rm {
        /// Domain name
        domain: String,
    },
}

#[derive(Subcommand)]
enum CertCommands {
    /// Import a TLS certificate
    Import {
        /// Domain name
        domain: String,
        /// Path to certificate PEM file
        #[arg(long)]
        cert: String,
        /// Path to private key PEM file
        #[arg(long)]
        key: String,
    },
}

#[derive(Subcommand)]
enum SetupComponent {
    /// Download and install standard CNI plugins
    Cni {
        /// Directory to install CNI plugin binaries
        #[arg(long, default_value = "/var/lib/helyos/cni/bin")]
        bin_dir: String,

        /// CNI plugins version to download
        #[arg(long, default_value = "1.4.1")]
        version: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    output::set_json_mode(cli.json);

    // Respect NO_COLOR (https://no-color.org/)
    if std::env::var_os("NO_COLOR").is_some() {
        console::set_colors_enabled(false);
        console::set_colors_enabled_stderr(false);
    }

    // Resolve server/token with precedence: CLI flag > env var > config file > default.
    // clap has already merged the CLI flag and env var into cli.server / cli.token
    // (these are None only when neither was provided).
    let file_cfg = config::load();
    let server: String = cli
        .server
        .clone()
        .or(file_cfg.server)
        .unwrap_or_else(|| "http://localhost:6443".to_string());
    let token: Option<String> = cli.token.clone().or(file_cfg.token);

    // Handle completions before validating server URL (completions don't need a server)
    if let Commands::Completions { shell } = &cli.command {
        let mut cmd = Cli::command();
        clap_complete::generate(*shell, &mut cmd, "helyos", &mut std::io::stdout());
        return Ok(());
    }

    // Validate --server URL
    if reqwest::Url::parse(&server).is_err() {
        output::print_error(&format!(
            "Invalid server URL: {}\n  Expected format: http(s)://host:port",
            server,
        ));
        std::process::exit(1);
    }

    if server.starts_with("http://")
        && server != "http://localhost:6443"
        && server != "http://127.0.0.1:6443"
    {
        eprintln!(
            "Warning: communicating over unencrypted HTTP. Secrets and tokens may be exposed."
        );
        eprintln!("  Use --server https://... for production environments.\n");
    }

    let client = client::HelyosClient::new(&server, token.as_deref());

    let result = match cli.command {
        Commands::Init { name, image } => commands::init(name.as_deref(), image.as_deref()),
        Commands::Deploy { file, timeout } => commands::deploy(&client, &file, timeout).await,
        Commands::Status => commands::status(&client).await,
        Commands::Top => commands::top::top(client, &server, token.as_deref()).await,
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
        Commands::Rm { name, project, yes } => {
            if !yes {
                let prompt = format!("Are you sure you want to remove deployment '{name}'?");
                if !dialoguer::Confirm::new()
                    .with_prompt(prompt)
                    .default(false)
                    .interact()?
                {
                    return Ok(());
                }
            }
            commands::remove(&client, project.as_deref(), &name).await
        }
        Commands::Project { command } => match command {
            ProjectCommands::List => commands::list_projects(&client).await,
            ProjectCommands::Create { name } => commands::create_project(&client, &name).await,
            ProjectCommands::Suspend { name } => commands::suspend_project(&client, &name).await,
            ProjectCommands::Resume { name } => commands::resume_project(&client, &name).await,
            ProjectCommands::Delete { name, yes } => {
                if !yes {
                    let prompt = format!(
                        "Are you sure you want to delete project '{name}' and all its resources?"
                    );
                    if !dialoguer::Confirm::new()
                        .with_prompt(prompt)
                        .default(false)
                        .interact()?
                    {
                        return Ok(());
                    }
                }
                commands::delete_project(&client, &name).await
            }
        },
        Commands::Secret { command } => match command {
            SecretCommands::Set {
                name,
                value,
                project,
            } => {
                let secret_value = match value {
                    Some(v) => v,
                    None => {
                        if std::io::stdin().is_terminal() {
                            dialoguer::Password::new()
                                .with_prompt(format!("Value for secret '{name}'"))
                                .interact()?
                        } else {
                            let mut buf = String::new();
                            std::io::stdin().read_line(&mut buf)?;
                            buf.trim_end().to_string()
                        }
                    }
                };
                commands::secret::set(&client, &project, &name, &secret_value).await
            }
            SecretCommands::List { project } => commands::secret::list(&client, &project).await,
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
        Commands::Routes { project } => commands::route::list(&client, project.as_deref()).await,
        Commands::Route { command } => match command {
            RouteCommands::Add {
                domain,
                project,
                deployment,
                https,
            } => commands::route::add(&client, &domain, &project, &deployment, https).await,
            RouteCommands::Rm { domain } => commands::route::remove(&client, &domain).await,
        },
        Commands::Cert { command } => match command {
            CertCommands::Import { domain, cert, key } => {
                commands::route::import_cert(&client, &domain, &cert, &key).await
            }
        },
        Commands::Setup { component } => match component {
            SetupComponent::Cni { bin_dir, version } => {
                commands::setup::cni(&bin_dir, &version).await
            }
        },
        Commands::Completions { .. } => unreachable!("handled above"),
    };

    if let Err(e) = result {
        let is_connect_error = e
            .downcast_ref::<reqwest::Error>()
            .is_some_and(|re| re.is_connect() || re.is_timeout());
        if is_connect_error {
            output::print_error_with_hint(
                "Cannot connect to helyosd",
                &format!(
                    "Is helyosd running? Start it with: helyosd --host {}",
                    server
                ),
            );
        } else {
            output::print_error(&e.to_string());
        }
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_deploy_command() {
        let cli = Cli::try_parse_from(["helyos", "deploy", "app.yaml"]).unwrap();
        match cli.command {
            Commands::Deploy { file, timeout } => {
                assert_eq!(file, "app.yaml");
                assert_eq!(timeout, 60);
            }
            _ => panic!("expected Deploy command"),
        }
    }

    #[test]
    fn parse_scale_command() {
        let cli = Cli::try_parse_from(["helyos", "scale", "api", "5", "-p", "myapp"]).unwrap();
        match cli.command {
            Commands::Scale {
                name,
                replicas,
                project,
            } => {
                assert_eq!(name, "api");
                assert_eq!(replicas, 5);
                assert_eq!(project, Some("myapp".to_string()));
            }
            _ => panic!("expected Scale command"),
        }
    }

    #[test]
    fn parse_pods_with_project() {
        let cli = Cli::try_parse_from(["helyos", "pods", "--project", "web"]).unwrap();
        match cli.command {
            Commands::Pods { project } => assert_eq!(project, Some("web".to_string())),
            _ => panic!("expected Pods command"),
        }
    }

    #[test]
    fn parse_json_flag() {
        let cli = Cli::try_parse_from(["helyos", "--json", "status"]).unwrap();
        assert!(cli.json);
    }

    #[test]
    fn parse_server_flag() {
        let cli =
            Cli::try_parse_from(["helyos", "--server", "http://10.0.1.1:6443", "status"]).unwrap();
        assert_eq!(cli.server.as_deref(), Some("http://10.0.1.1:6443"));
    }

    #[test]
    fn parse_secret_set() {
        let cli = Cli::try_parse_from([
            "helyos", "secret", "set", "DB_PASS", "--value", "s3cret", "-p", "myapp",
        ])
        .unwrap();
        match cli.command {
            Commands::Secret { command } => match command {
                SecretCommands::Set {
                    name,
                    value,
                    project,
                } => {
                    assert_eq!(name, "DB_PASS");
                    assert_eq!(value, Some("s3cret".to_string()));
                    assert_eq!(project, "myapp");
                }
                _ => panic!("expected Set"),
            },
            _ => panic!("expected Secret"),
        }
    }

    #[test]
    fn parse_route_add() {
        let cli = Cli::try_parse_from([
            "helyos",
            "route",
            "add",
            "api.example.com",
            "-p",
            "web",
            "--deployment",
            "api",
            "--https",
        ])
        .unwrap();
        match cli.command {
            Commands::Route { command } => match command {
                RouteCommands::Add {
                    domain,
                    project,
                    deployment,
                    https,
                } => {
                    assert_eq!(domain, "api.example.com");
                    assert_eq!(project, "web");
                    assert_eq!(deployment, "api");
                    assert!(https);
                }
                _ => panic!("expected Route Add subcommand"),
            },
            _ => panic!("expected Route command"),
        }
    }
}
