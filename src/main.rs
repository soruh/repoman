// #[macro_use]
// extern crate clap;
// use clap::App;

extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;

mod settings;

use settings::Settings;

fn main() {
    let settings = Settings::new();

    // Print out our settings
    println!("{:?}", settings);

    /*
    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml)
        // .setting(clap::AppSettings::ArgRequiredElseHelp)
        // .setting(clap::AppSettings::SubcommandRequired)
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let (name, args) = matches.subcommand();

    if let Some(args) = args {
        println!("args: {:#?}", args);

        match name {
            "templates" => templates(args),
            "init" => init(args),
            "convert" => convert(args),
            _ => panic!("recieved unexpected subcommand"),
        }
    } else {
        panic!("missing arguments");
    }
    */
}

/*
fn templates(args: &clap::ArgMatches) {
    //
}
fn init(args: &clap::ArgMatches) {
    //
}
fn convert(args: &clap::ArgMatches) {
    //
}
*/
