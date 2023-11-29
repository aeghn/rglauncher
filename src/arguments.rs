use clap::Parser;
use crate::constant;

#[derive(Parser, Default, Debug)]
#[command(author = constant::PROJECT_AUTHOR, version = constant::PROJECT_VERSION, about = constant::PROJECT_DESCRIPTION)]
pub struct Arguments {
    #[clap(long, help = "Gtk Theme to Use")]
    pub theme: String,
    #[clap(long, help = "The path of mdict files, including css files.")]
    pub dict_dir: String,
    #[clap(long, help="The file path of clipboard db.")]
    pub clip_db: String,
}

