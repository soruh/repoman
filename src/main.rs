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
    // TODO: check if git, cp, ssh etc. exist
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

fn create_ssh_remote(settings: &Settings, repo_name: &str) -> Result<(), String> {
    // * ssh: run "git init [path]" on [host] as [user]

    let mut remote_repo_path = settings.ssh_remote_repo_path.clone();
    if !remote_repo_path.ends_with("/") {
        remote_repo_path.push_str("/");
    }
    remote_repo_path.push_str(&repo_name);
    if settings.ssh_remote_add_git_suffix {
        remote_repo_path.push_str(".git");
    }

    let mut command = String::from("git init ");
    if settings.ssh_remote_use_bare {
        command.push_str("--bare ");
    }
    command.push_str("\"");
    command.push_str(&remote_repo_path);
    command.push_str("\"");

    // println!("running command {} on remote", command);

    let mut child = Command::new("ssh")
        .args(&[&settings.ssh_remote_host, &command])
        .spawn()
        .map_err(|err| format!("failed to start git init on remote:\n{:?}", err))?;

    let status = child
        .wait()
        .map_err(|_| "awaiting git init on remote failed")?;

    if !status.success() {
        return Err(format!(
            "git init failed with code: {}",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

fn init(settings: &Settings, args: &clap::ArgMatches) -> Result<(), String> {
    // println!(
    //     "\x1b[32mcommand\x1b[0m: {}\n\x1b[33msettings\x1b[0m: {:#?}\n\x1b[34margs\x1b[0m: {:#?}",
    //     "init", settings, args.args
    // );

    // TODO: allow ssh dirs

    let repo_name = args.value_of("name").unwrap();

    let mut new_repo_dir = settings.repo_path.clone();
    if !new_repo_dir.ends_with("/") {
        new_repo_dir.push_str("/");
    }
    new_repo_dir.push_str(repo_name.as_ref());
    new_repo_dir.push_str("/");

    // println!("path for new repo: {}", new_repo_dir);

    if Path::new(&new_repo_dir).exists() {
        return Err(format!("repo at {:?} already exists", new_repo_dir));
    }

    let mut applied_template = false;

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
                .args(&["-HTr", &template_path, &new_repo_dir])
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
                    status.code().unwrap_or(-1)
                ));
            }
        }

        applied_template = true;
    }

    let mut git_path = new_repo_dir.clone();
    git_path.push_str(".git");

    if Path::new(&git_path).exists() {
        println!("Folder already contains a repo, so we don't need to create one.");
    } else {
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
    }

    if applied_template {
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
    }

    println!("creating initial commit");
    let mut child = Command::new("git")
        .args(&["commit", "--allow-empty", "-m", "initial commit"])
        .current_dir(&new_repo_dir)
        .spawn()
        .map_err(|err| format!("failed to start git commit:\n{:?}", err))?;

    child.wait().map_err(|_| "awaiting git commit failed")?;

    if settings.use_ssh_remote {
        create_ssh_remote(settings, repo_name.as_ref())?;

        let mut remote_location = settings.ssh_remote_host.clone();
        remote_location.push(':');
        remote_location.push_str(&settings.ssh_remote_repo_path);
        if !remote_location.ends_with("/") {
            remote_location.push_str("/");
        }
        remote_location.push_str(repo_name.as_ref());

        println!("adding ssh remote...");
        let mut child = Command::new("git")
            .args(&["remote", "add", "origin", &remote_location])
            .current_dir(&new_repo_dir)
            .spawn()
            .map_err(|err| format!("failed to start git commit:\n{:?}", err))?;

        child
            .wait()
            .map_err(|err| format!("awaiting git remote add failed:\n{:?}", err))?;

        println!("pushing to ssh remote...");
        let mut child = Command::new("git")
            .args(&["push", "--set-upstream", "origin", "master"])
            .current_dir(&new_repo_dir)
            .spawn()
            .map_err(|err| format!("failed to start git commit:\n{:?}", err))?;

        child
            .wait()
            .map_err(|err| format!("awaiting git remote add failed:\n{:?}", err))?;
    }

    Ok(())
}
fn templates(_settings: &Settings, _args: &clap::ArgMatches) -> Result<(), String> {
    unimplemented!();
}
fn convert(_settings: &Settings, _args: &clap::ArgMatches) -> Result<(), String> {
    unimplemented!();
}
