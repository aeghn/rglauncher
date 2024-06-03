use crate::constants;
use clap::Parser;

#[derive(Parser, Default, Debug, Clone)]
#[command(author = constants::PROJECT_AUTHOR, version = constants::PROJECT_VERSION, about = constants::PROJECT_DESCRIPTION)]
pub struct Arguments {
    #[clap(long, help = "The file path of config file.")]
    pub config_file: Option<String>,
}
