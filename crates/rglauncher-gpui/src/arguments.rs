use clap::Parser;

#[derive(Parser, Default, Debug, Clone)]

pub struct Arguments {
    #[clap(long, help = "The file path of config file.")]
    pub config_file: Option<String>,
}
