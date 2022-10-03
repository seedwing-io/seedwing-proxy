use clap::builder::PathBufValueParser;
use clap::{value_parser, Arg, Command};

pub const COMMAND_NAME: &str = "seedwing-proxy";

pub fn cli() -> Command {
    Command::new(COMMAND_NAME)
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .value_name("path of configuration [seedwing.toml]")
                .value_parser(PathBufValueParser::default()),
        )
        .arg(
            Arg::new("bind")
                .long("bind")
                .short('b')
                .value_name("bind address"),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .short('p')
                .value_name("listen port")
                .value_parser(value_parser!(u16)),
        )
}
