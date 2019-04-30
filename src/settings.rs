use config::{Config, File};
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub config_path: String,

    pub repo_path: String,

    pub use_ssh_remote: bool,
    pub ssh_remote_host: String,
    pub ssh_remote_repo_path: String,
    pub ssh_remote_use_bare: bool,
    pub ssh_remote_add_git_suffix: bool,
}

fn load_config_path(s: &mut Config, config_path: &str) -> Result<(), String> {
    if let Err(err) = s.set("config_path", config_path.as_ref()) {
        return Err(format!("failed to set config_path: \n{:#?}", err));
    }

    let mut config_folder = String::from(config_path);
    config_folder.push_str("config.toml");

    if let Err(error) = s.merge(File::with_name(config_folder.as_ref())) {
        return Err(format!("failed to load config:\n{:#?}", error));
    }

    Ok(())
}

fn copy_default_config(config_path: &str) -> Result<(), String> {
    let exe_path = env::current_exe().expect("failed to get executable path");
    let default_config_path = exe_path
        .parent()
        .unwrap()
        .to_path_buf()
        .join("../../default.config")
        .canonicalize()
        .expect("failed to normalize default config path");
    fs::create_dir_all(&config_path).expect("failed to mkdir");

    let output = Command::new("cp")
        .args(&["-HTr", &default_config_path.to_string_lossy(), config_path])
        .output()
        .expect("failed to copy default config");

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "failed to copy default config:\n {:?}",
            output.stderr
        ))
    }
}

impl Settings {
    pub fn insert_home_dir(&mut self) -> Result<(), String> {
        let home = match env::var("HOME") {
            Ok(home) => Some(home),
            Err(_) => None,
        };

        loop {
            //TODO: expand to all paths
            // since repo_path is the only one for now, this is fine

            if self.repo_path.find("~") == None {
                continue;
            }

            match home {
                None => {
                    return Err(format!(
                        "a configured path references '~', but $HOME is not set"
                    ))
                }
                Some(home) => {
                    self.repo_path = self.repo_path.replace("~", home.as_ref());
                }
            }

            break;
        }
        Ok(())
    }

    pub fn new<'a>() -> Result<Self, String> {
        let mut config_paths = vec!["/etc/repoman/"];

        let mut path1 = String::new();
        let mut path2 = String::new(); //TODO: can this be inside of the if scope somehow?

        let home = env::var("HOME");
        if let Ok(home) = home {
            path1.push_str(home.as_ref());
            path1.push_str("/.repoman/");
            config_paths.insert(0, path1.as_ref());

            path2.push_str(home.as_ref());
            path2.push_str("/.config/repoman/");
            config_paths.insert(0, path2.as_ref());
        }

        let mut config_path: Option<String> = None;

        for path in &config_paths {
            if let Ok(metadata) = fs::metadata(path) {
                if metadata.is_dir() {
                    config_path = Some(path.to_string());
                }
            }
        }

        let mut s = Config::new();

        if let Some(config_path) = config_path {
            load_config_path(&mut s, config_path.as_ref())?;
        } else {
            eprintln!("no config found.");
            eprintln!("please choose one of the following locations to create one:");

            for i in 0..config_paths.len() {
                eprintln!("{}: {}", i, config_paths[i]);
            }

            loop {
                eprint!("> ");

                io::stderr().flush().ok();

                let mut path_num = String::new();
                io::stdin()
                    .read_line(&mut path_num)
                    .expect("Failed to read line");

                let path_num: usize = match path_num.trim().parse() {
                    Ok(num) => num,
                    Err(_) => continue,
                };

                if path_num >= config_paths.len() {
                    continue;
                }

                config_path = Some(config_paths[path_num].to_string());
                break;
            }
            let config_path = config_path.unwrap();

            copy_default_config(config_path.as_ref())?;
            load_config_path(&mut s, &config_path.as_ref())?;
        }

        s.try_into()
            .map_err(|error| format!("configuration error:\n{:#?}", error))
    }
}
