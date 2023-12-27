use crate::constants;
use clap::Parser;

#[derive(Parser, Default, Debug, Clone)]
#[command(author = constants::PROJECT_AUTHOR, version = constants::PROJECT_VERSION, about = constants::PROJECT_DESCRIPTION)]
pub struct Arguments {
    #[clap(long, help = "Icon Theme to Use")]
    pub icon: String,
    #[clap(long, help = "The path of mdict files, including css files.")]
    pub dict_dir: String,
    #[clap(long, help = "The file path of clipboard db.")]
    pub clip_db: String,
}
