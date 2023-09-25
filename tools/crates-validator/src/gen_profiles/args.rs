#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to existing database on disk
    ///
    /// If not supplied, will re-download to /tmp
    #[arg(short, long)]
    pub(crate) db_path: Option<std::path::PathBuf>,

    /// Sets the output location of the json [ProfileConfig]
    #[arg(short, long)]
    pub(crate) output_path: std::path::PathBuf,

    /// Sets the number of crates, sorted by downloads to profile
    #[arg(short, long, default_value = "20")]
    pub(crate) crate_count: usize,

    /// Single crate mode
    ///
    /// Will only create a profile for the single crate provide
    #[arg(short, long, conflicts_with = "crate_count")]
    pub(crate) name: Option<String>,

    /// Disable profiles
    ///
    /// This will disable all crate-specific profile modifications
    #[arg(long, conflicts_with = "profiles_file")]
    no_profiles: bool,

    /// Specify the path for the configuration file containing custom
    /// instructions on how to generate the profile for crates.
    #[arg(short, long, conflicts_with = "no_profiles")]
    pub(crate) profiles_file: Option<String>,

    /// Add selected categories to the profile
    #[arg(
        short = 'C',
        long,
        conflicts_with = "name",
        value_parser,
        value_delimiter = ' '
    )]
    pub(crate) categories: Option<Vec<String>>,

    /// Number of crates per category to add to the profile
    ///
    /// This will add the top N crates per category to the profile, sorted by
    /// downloads.
    #[arg(short = 'L', long, default_value = None, requires = "categories")]
    pub(crate) category_count_limit: Option<usize>,
}
