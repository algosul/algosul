use std::{
    borrow::Cow,
    ffi::OsString,
    fmt::{Debug, Display, Formatter},
    io,
    path::{Path, PathBuf},
};

use clap::{value_parser, CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{aot::generate, Generator};
use colored::Colorize;
use io::Write;
#[derive(Debug, )]
pub enum Error {
    IOError(io::Error),
    NoHomeDirError,
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_, >) -> std::fmt::Result {
        match self {
            Error::IOError(e, ) => write!(f, "IO error: {e}"),
            Error::NoHomeDirError => write!(f, "No home directory found"),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static), > {
        match self {
            Error::IOError(e, ) => Some(e),
            Error::NoHomeDirError => None,
        }
    }
}
pub type Result<T, > = std::result::Result<T, Error, >;
#[derive(Debug, Clone, PartialEq, Eq, Parser, )]
#[command(version, about, author, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    sub_command: SubCommand,
}
pub use clap_complete::Shell;
#[derive(Debug, Clone, PartialEq, Eq, Parser, )]
pub struct ShellOptions {
    /// the shell to generate the auto-completion script for
    #[clap(value_parser = value_parser!(Shell))]
    shell: Shell,
}
#[derive(Debug, Clone, PartialEq, Eq, Subcommand, )]
pub enum AutoCompleteSubCommand {
    /// install auto-completion script
    Install {
        #[clap(flatten)]
        shell: ShellOptions,
    },
    /// reinstall auto-completion script
    Reinstall {
        #[clap(flatten)]
        shell: ShellOptions,
    },
    /// remove auto-completion script
    Remove {
        #[clap(flatten)]
        shell: ShellOptions,
    },
    /// output auto-completion script
    Output {
        #[clap(flatten)]
        shell: ShellOptions,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Subcommand, )]
pub enum SubCommand {
    /// auto-completion script
    AutoComplete {
        #[clap(subcommand)]
        sub_command: AutoCompleteSubCommand,
    },
    /// initialize a new workspace or project
    Init {
        #[clap(flatten)]
        scope: ScopeOptions,
    },
    /// create a new workspace or project or profile
    Create {
        #[clap(flatten)]
        scope: ScopeOptions,
    },
    /// remove a workspace or project or profile
    Remove {
        #[clap(flatten)]
        scope: ScopeOptions,
    },
    /// undo the last command in workspace or project or profile
    Undo {
        #[clap(flatten)]
        scope: ScopeOptions,
    },
    /// redo the last command in workspace or project or profile
    Redo {
        #[clap(flatten)]
        scope: ScopeOptions,
    },
    /// build workspace or project or profile
    Build {
        /// the binary to build (default: build all binaries)
        #[clap(short, long)]
        binary: Option<String, >,
        #[clap(flatten)]
        scope: ScopeOptions,
    },
    /// clean workspace or project or profile
    Clean {
        #[clap(flatten)]
        scope: ScopeOptions,
    },
    /// run binary in a workspace or project or profile
    Run {
        /// the binary to run (default: run all binaries)
        #[clap(short, long)]
        binary: Option<String, >,
        /// the arguments to pass to the binary
        #[clap(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<OsString, >,
        #[clap(flatten)]
        scope: ScopeOptions,
    },
    /// rebuild workspace or project or profile
    Rebuild {
        /// the binary to rebuild (default: rebuild all binaries)
        #[clap(short, long)]
        binary: Option<String, >,
        #[clap(flatten)]
        scope: ScopeOptions,
    },
}
/// Options for the scope of the command (workspace, project, profile)
#[derive(Default, Debug, Clone, PartialEq, Eq, Parser, )]
pub struct ScopeOptions {
    /// set the workspace directory
    #[clap(short, long, value_hint = ValueHint::DirPath)]
    pub workspace: Option<PathBuf, >,
    /// set the project name
    #[clap(short = 'j', long, value_hint = ValueHint::DirPath)]
    pub project: Option<String, >,
    /// set the profile name
    #[clap(short, long, value_hint = ValueHint::Unknown)]
    pub profile: Option<String, >,
}
pub fn parse_args() -> Cli { Cli::parse() }
pub fn generate_completion<G: Generator, >(generator: G, bin_name: &str, buf: &mut dyn Write) {
    generate(generator, &mut Cli::command(), bin_name, buf);
}
impl Cli {
    pub fn run(&self) -> Result<(), > { self.sub_command().run() }

    pub fn sub_command(&self) -> &SubCommand { &self.sub_command }

    pub fn sub_command_mut(&mut self) -> &mut SubCommand { &mut self.sub_command }
}
impl ShellOptions {
    pub fn config_dir(&self) -> Result<Cow<'static, Path, >, > {
        match self.shell {
            Shell::Bash => Ok(Cow::Borrowed(Path::new("/etc/bash_completion.d"))),
            Shell::Zsh => Ok(Cow::Borrowed(Path::new("/usr/local/share/zsh/site-functions"))),
            Shell::Fish => Ok(Cow::Borrowed(Path::new("/usr/share/fish/vendor_completions.d"))),
            Shell::PowerShell => {
                Ok(Cow::Borrowed(Path::new("/usr/local/share/powershell/Modules/")))
            }
            Shell::Elvish => Ok(Cow::Borrowed(Path::new("/usr/share/elvish/lib/"))),
            shell => panic!("unsupported shell: {shell}"),
        }
    }

    pub fn config_file_name(&self) -> Cow<'static, Path, > {
        let app_name = crate::NAME;
        match self.shell {
            Shell::Bash => Cow::Borrowed(Path::new(app_name)),
            Shell::Zsh => Cow::Owned(format!("_{app_name}").into()),
            Shell::Fish => Cow::Owned(format!("{app_name}.fish").into()),
            Shell::PowerShell => Cow::Owned(format!("{app_name}.ps1").into()),
            Shell::Elvish => Cow::Owned(format!("_{app_name}.elv").into()),
            shell => panic!("unsupported shell: {shell}"),
        }
    }

    pub fn config_file_path(&self) -> Result<Cow<'static, Path, >, > {
        let config_dir = self.config_dir()?;
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir).map_err(Error::IOError)?;
        }
        Ok(Cow::Owned(config_dir.join(self.config_file_name())))
    }

    pub fn show_installed_info(&self, config_file_path: Cow<Path, >) {
        match self.shell {
            Shell::PowerShell => {
                println!(
                    "{}: Please run 'Import-Module \"{}\"' in powershell",
                    "INFO".bright_white(),
                    config_file_path.display()
                );
            }
            shell @ (Shell::Bash | Shell::Zsh | Shell::Fish) => {
                println!(
                    "{}: Please reset {1}, or run 'source \"{2}\"' in {1}",
                    "INFO".bright_white(),
                    shell,
                    config_file_path.display()
                );
            }
            Shell::Elvish => {
                println!(
                    "{}: Please reset {1}, or run '{1} \"{2}\"', \n\tFor more information, see https://github.com/zzamboni/elvish-completions.",
                    "INFO".bright_white(),
                    Shell::Elvish,
                    config_file_path.display(),
                );
            }
            shell => panic!("unsupported shell: {shell}"),
        }
    }
}
impl SubCommand {
    pub fn run(&self) -> Result<(), > {
        match self {
            SubCommand::AutoComplete { sub_command, } => {
                let mut buffer = Vec::new();
                match sub_command {
                    AutoCompleteSubCommand::Output { shell, } => {
                        generate_completion(shell.shell, &crate::NAME, &mut buffer);
                        io::stdout().write_all(&buffer).map_err(Error::IOError)?;
                        Ok(())
                    }
                    AutoCompleteSubCommand::Install { shell, } => {
                        let config_file_path = shell.config_file_path()?;
                        println!(
                            "the auto-completion script for {} will be installed in '{}'.",
                            shell.shell,
                            config_file_path.display()
                        );
                        {
                            let mut config_file =
                                match std::fs::File::create_new(&config_file_path) {
                                    Err(e, ) if e.kind() == io::ErrorKind::AlreadyExists => {
                                        println!(
                                            "{}: the auto-completion script for {} is already \
                                             installed.",
                                            "WARNING".bright_yellow(),
                                            shell.shell
                                        );
                                        return Ok(());
                                    }
                                    res => res,
                                }
                                    .map_err(Error::IOError)?;
                            generate_completion(shell.shell, &crate::NAME, &mut config_file);
                        }
                        println!(
                            "the auto-completion script for {} was installed {}.",
                            shell.shell,
                            "successfully".bright_green()
                        );
                        shell.show_installed_info(config_file_path);
                        Ok(())
                    }
                    AutoCompleteSubCommand::Reinstall { shell, } => {
                        let config_file_path = shell.config_file_path()?;
                        if config_file_path.exists() {
                            println!(
                                "the auto-completion script for {} will be reinstalled in '{}'.",
                                shell.shell,
                                config_file_path.display()
                            );
                            std::fs::remove_file(&config_file_path).map_err(Error::IOError)?;
                            {
                                let mut config_file = std::fs::File::create(&config_file_path)
                                    .map_err(Error::IOError)?;
                                generate_completion(
                                    shell.shell,
                                    &crate::NAME,
                                    &mut config_file,
                                );
                            }
                            println!(
                                "the auto-completion script for {} was reinstalled {}.",
                                shell.shell,
                                "successfully".bright_green()
                            );
                            shell.show_installed_info(config_file_path);
                            Ok(())
                        } else {
                            println!(
                                "{}: the auto-completion script for {} was not installed in '{}'.",
                                "ERROR".bright_red(),
                                shell.shell,
                                config_file_path.display()
                            );
                            std::process::exit(1);
                        }
                    }
                    AutoCompleteSubCommand::Remove { shell, } => {
                        let config_file_path = shell.config_file_path()?;
                        if config_file_path.exists() {
                            println!(
                                "the auto-completion script for {} will be removed from '{}'.",
                                shell.shell,
                                config_file_path.display()
                            );
                            std::fs::remove_file(&config_file_path).map_err(Error::IOError)?;
                            println!(
                                "the auto-completion script for {} was removed {}.",
                                shell.shell,
                                "successfully".bright_green()
                            );
                            Ok(())
                        } else {
                            println!(
                                "{}: the auto-completion script for {} was not installed in '{}'.",
                                "ERROR".bright_red(),
                                shell.shell,
                                config_file_path.display()
                            );
                            std::process::exit(1);
                        }
                    }
                }
            }
            SubCommand::Init { scope: _scope, } => {
                todo!()
            }
            SubCommand::Create { scope: _scope, } => {
                todo!()
            }
            SubCommand::Remove { scope: _scope, } => {
                todo!()
            }
            SubCommand::Undo { scope: _scope, } => {
                todo!()
            }
            SubCommand::Redo { scope: _scope, } => {
                todo!()
            }
            SubCommand::Build { binary: _binary, scope: _scope, } => {
                todo!()
            }
            SubCommand::Clean { scope: _scope, } => {
                todo!()
            }
            SubCommand::Run { binary: _binary, args: _args, scope: _scope, } => {
                todo!()
            }
            SubCommand::Rebuild { binary: _binary, scope: _scope, } => {
                todo!()
            }
        }
    }
}