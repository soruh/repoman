#[macro_use]
extern crate clap;
use clap::App;

extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;

mod settings;

use settings::Settings;
use std::path::Path;
use std::process::Command;

fn check_prerequisites() -> Result<(), String> {
    // TODO: check if git, cp etc. exist
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("\x1b[31mAn error occured:\x1b[0m\n{}", err);
    }
}

fn run() -> Result<(), String> {
    check_prerequisites()?;

    let mut settings = Settings::new()?;
    settings.insert_home_dir()?;

    std::fs::create_dir_all(&settings.repo_path).map_err(|_| "failed to create repository dir")?;

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
            _ => return Err(String::from("recieved unexpected subcommand")),
        }
    } else {
        return Err(String::from("no arguments found"));
    }
}

fn init(settings: &Settings, args: &clap::ArgMatches) -> Result<(), String> {
    // println!(
    //     "\x1b[32mcommand\x1b[0m: {}\n\x1b[33msettings\x1b[0m: {:#?}\n\x1b[34margs\x1b[0m: {:#?}",
    //     "init", settings, args.args
    // );

    // TODO: allow ssh dirs

    let mut new_repo_dir = settings.repo_path.clone();
    new_repo_dir.push_str("/");
    new_repo_dir.push_str(args.value_of("name").unwrap());
    new_repo_dir.push_str("/");

    // println!("path for new repo: {}", new_repo_dir);

    if Path::new(&new_repo_dir).exists() {
        return Err(format!("repo at {:?} already exists", new_repo_dir));
    }

    if let Some(template) = args.value_of("template") {
        println!("template: {}", template);

        let mut template_path = settings.config_path.clone();
        template_path.push_str("templates/");
        template_path.push_str(template);

        println!("template_path: {}", template_path);

        let is_dir = match std::fs::metadata(&template_path) {
            Err(err) => {
                return Err(format!(
                    "failed to access template {} at {}:\n {}",
                    template, template_path, err
                ))
            }
            Ok(metadata) => metadata.is_dir(),
        };

        if is_dir {
            println!("copying template directory");
            let output = Command::new("cp")
                .args(&["-Tr", &template_path, &new_repo_dir])
                .output()
                .map_err(|_| "failed to copy template")?;
            if !output.status.success() {
                return Err(format!(
                    "failed to copy template directory:\n {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        } else {
            println!("executing template script...");
            let mut child = Command::new(&template_path)
                .args(&[&new_repo_dir, &template_path])
                .spawn()
                .map_err(|err| format!("setup script failed to execute:\n{:?}", err))?;

            let status = child.wait().map_err(|_| "awaiting setup script failed")?;
            println!("done");
            if !status.success() {
                return Err(format!(
                    "setup script failed with code: {}",
                    status.code().unwrap_or(1)
                ));
            }
        }
    }

    let mut git_path = new_repo_dir.clone();
    git_path.push_str(".git");

    if !Path::new(&git_path).exists() {
        println!("initializing new repo...");
        let mut child = Command::new("git")
            .args(&["init", &new_repo_dir])
            .spawn()
            .map_err(|err| format!("failed to start git init:\n{:?}", err))?;

        let status = child.wait().map_err(|_| "awaiting git init failed")?;
        if !status.success() {
            return Err(format!(
                "git init failed with code: {}",
                status.code().unwrap_or(1)
            ));
        }

        println!("adding template files to git...");
        let mut child = Command::new("git")
            .args(&["add", "-A"])
            .current_dir(&new_repo_dir)
            .spawn()
            .map_err(|err| format!("failed to start git add:\n{:?}", err))?;

        let status = child.wait().map_err(|_| "awaiting git add failed")?;
        if !status.success() {
            return Err(format!(
                "git add failed with code: {}",
                status.code().unwrap_or(1)
            ));
        }

        println!("commiting template files...");
        let mut child = Command::new("git")
            .args(&["commit", "-m", "commit template"])
            .current_dir(&new_repo_dir)
            .spawn()
            .map_err(|err| format!("failed to start git commit:\n{:?}", err))?;

        // let status =
        child.wait().map_err(|_| "awaiting git commit failed")?;
        // if !status.success() {
        //     return Err(format!(
        //         "git commit failed with code: {}",
        //         status.code().unwrap_or(1)
        //     ));
        // }
    }

    // ? 5. if remote configured:
    //      * setup remote

    Ok(())
}
fn templates(_settings: &Settings, _args: &clap::ArgMatches) -> Result<(), String> {
    unimplemented!();
}
fn convert(_settings: &Settings, _args: &clap::ArgMatches) -> Result<(), String> {
    unimplemented!();
}
