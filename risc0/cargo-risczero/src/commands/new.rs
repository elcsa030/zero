use std::path::PathBuf;

use cargo_generate::{GenerateArgs, TemplatePath, Vcs};
use convert_case::{Case, Casing};

const RISC0_GH_REPO: &str = "https://github.com/risc0/risc0";
const RICS0_TEMPLATE_DIR: &str = "templates/rust-stater";
const RISC0_RELEASE_TAG: &str = "v0.13.0";

#[derive(clap::Args)]
/// `cargo risczero new`
pub struct NewCommand {
    /// Name which will be used as the output project name.
    #[arg()]
    pub name: String,

    /// GH repository name.
    #[clap(value_parser, long, short, default_value = RISC0_GH_REPO)]
    pub template: String,

    /// template git tag.
    #[clap(value_parser, long, default_value = RISC0_RELEASE_TAG)]
    pub tag: String,

    /// template git branch, overrides `tag` option
    #[clap(value_parser, long, default_value = "")]
    pub branch: String,

    /// Destination directory to create project in.
    ///
    /// Example `cargo risczero new --dest /tmp/ cool-project` ->
    /// `/tmp/cool-project/`
    ///
    /// Default: `pwd`
    #[clap(value_parser, long)]
    pub dest: Option<PathBuf>,

    /// Disable init'ing a git repo in the dest project
    #[clap(value_parser, long, global = true)]
    pub no_git: bool,
}

impl NewCommand {
    pub fn run(&self) {
        let subfolder = if self.template == RISC0_GH_REPO {
            Some(RICS0_TEMPLATE_DIR.to_string())
        } else {
            None
        };

        let dest_dir = if let Some(dest_dir) = self.dest.clone() {
            dest_dir
        } else {
            std::env::current_dir()
                .expect("Failed to fetch cwd")
                .join(self.name.clone())
        };

        let mut template_path = TemplatePath {
            auto_path: Some(self.template.clone()),
            subfolder,
            git: None,
            branch: None,
            path: None,
            favorite: None,
            tag: Some(self.tag.clone()),
            test: false,
        };

        if !self.branch.is_empty() {
            template_path.branch = Some(self.branch.clone());
            template_path.tag = None;
        }

        cargo_generate::generate(GenerateArgs {
            template_path,
            list_favorites: false,
            name: Some(self.name.to_case(Case::Snake)),
            force: true,
            verbose: false,
            template_values_file: None,
            silent: false,
            config: None,
            vcs: if self.no_git {
                Some(Vcs::None)
            } else {
                Some(Vcs::Git)
            },
            lib: false,
            bin: false,
            ssh_identity: None,
            define: vec![],
            init: false,
            destination: Some(dest_dir),
            force_git_init: false,
            allow_commands: false,
            overwrite: false,
            other_args: None,
        })
        .expect("Failed to generate project");
    }
}
