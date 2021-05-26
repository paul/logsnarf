use clap::Clap;

#[derive(Clap, Debug)]
pub struct ParseOpt {
    #[clap(short = 'f', long)]
    pub file: Option<String>,

    #[clap(short = 'D', long)]
    pub tsdb_url: Option<String>,
}

#[derive(Clap, Debug)]
pub struct ServerOpt {
    #[clap(short = 'p', long)]
    pub port: Option<i32>,

    #[clap(short = 'D', long)]
    pub database_url: Option<String>,
}
