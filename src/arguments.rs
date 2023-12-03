use clap::Parser;
use crate::constants;

#[derive(Parser, Default, Debug, Clone)]
#[command(author = constants::PROJECT_AUTHOR, version = constants::PROJECT_VERSION, about = constants::PROJECT_DESCRIPTION)]
pub struct Arguments {
    #[clap(long, help = "Gtk Theme to Use")]
    pub theme: String,
    #[clap(long, help = "The path of mdict files, including css files.")]
    pub dict_dir: String,
    #[clap(long, help="The file path of clipboard db.")]
    pub clip_db: String,
}

