use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "zash")]
pub struct Opts {
    // These ops are just here to prevent errors with applications
    #[structopt(short, long)]
    pub interactive: bool,

    #[structopt(short, long)]
    pub login: bool,

    #[structopt(short, long)]
    pub command: Option<String>,

    #[structopt(max_values = 1)]
    pub script_file: Option<String>
}