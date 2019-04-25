#[macro_use]
extern crate clap;
use clap::App;

extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;

mod settings;

use settings::Settings;

fn main() {
    let mut settings = Settings::new();

    settings.insert_home_dir();

    // println!("{:?}", settings);

    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml)
        // .setting(clap::AppSettings::ArgRequiredElseHelp)
        // .setting(clap::AppSettings::SubcommandRequired)
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let (name, args) = matches.subcommand();

    if let Some(args) = args {
        // println!("args: {:#?}", args);

        match name {
            "templates" => templates(&settings, args),
            "init" => init(&settings, args),
            "convert" => convert(&settings, args),
            _ => panic!("recieved unexpected subcommand"),
        }
    } else {
        panic!("no arguments found");
    }
}

fn init(settings: &Settings, args: &clap::ArgMatches) {
    println!(
        "\x1b[32mcommand\x1b[0m: {}\n\x1b[33msettings\x1b[0m: {:#?}\n\x1b[34margs\x1b[0m: {:#?}",
        "init", settings, args.args
    );

    // TODO: allow ssh dirs

    // * 0. calculate repo dir
    // ? 1. if dir exists:
    //      ! abort
    // ? 2. if template ist set:
    //      * Either: copy template dir
    //      * Or    : run template script
    // ? 3. if .git doesn't exist:
    //      * git init [Folder]
    // ? 4. if file changes:
    //      * add & commit all changed files (-> template files)
    // ? 5. if remote configured:
    //      * setup remote
}
fn templates(settings: &Settings, args: &clap::ArgMatches) {
    println!(
        "\x1b[32mcommand\x1b[0m: {}\n\x1b[33msettings\x1b[0m: {:#?}\n\x1b[34margs\x1b[0m: {:#?}",
        "template", settings, args.args
    );
}
fn convert(settings: &Settings, args: &clap::ArgMatches) {
    println!(
        "\x1b[32mcommand\x1b[0m: {}\n\x1b[33msettings\x1b[0m: {:#?}\n\x1b[34margs\x1b[0m: {:#?}",
        "convert", settings, args.args
    );
}
