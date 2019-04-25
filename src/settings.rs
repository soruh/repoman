use config::{Config, File};
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct Settings {
    repo_path: String,
}

fn load_config_path(s: &mut Config, config_path: &str) {
    if let Err(err) = s.set("config_path", config_path.as_ref()) {
        eprintln!("failed to set config_path: \n{:#?}", err);
        std::process::exit(1);
    }

    let mut config_folder = String::from(config_path);
    config_folder.push_str("/config.toml");

    if let Err(error) = s.merge(File::with_name(config_folder.as_ref())) {
        eprintln!("failed to load config:\n{:#?}", error);
        std::process::exit(1);
    }
}

fn copy_default_config(config_path: &str) {
    let exe_path = env::current_exe().expect("failed to get executable path");
    let default_config_path = exe_path
        .parent()
        .unwrap()
        .to_path_buf()
        .join("../../default.config")
        .canonicalize()
        .expect("failed to normalize default config path");
    fs::create_dir_all(&config_path).expect("failed to mkdir");

    Command::new("cp")
        .args(&["-Tr", &default_config_path.to_string_lossy(), config_path])
        .output()
        .expect("failed to copy default config");
}

impl Settings {
    pub fn insert_home_dir(&mut self) {
        let home = match env::var("HOME") {
            Ok(home) => Some(home),
            Err(_) => None,
        };


        loop{ //TODO: expand to all paths
            // since repo_path is the only one for now, this is fine


            if self.repo_path.find("~") == None {
                continue;
            }

            // TODO: check if path is local (no method in "url")

            match home {
                None => panic!("a config path references '~', but $HOME is not set"),
                Some(home) => {
                    self.repo_path = self.repo_path.replace("~", home.as_ref());
                }
            }

            break;
        }
    }

    pub fn new<'a>() -> Self {
        let mut config_paths = vec!["/etc/repoman"];

        let mut path1 = String::new();
        let mut path2 = String::new(); //TODO: can this be inside of the if scope somehow?

        let home = env::var("HOME");
        if let Ok(home) = home {
            path1.push_str(home.as_ref());
            path1.push_str("/.repoman");
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
            load_config_path(&mut s, config_path.as_ref());
        } else {
            eprintln!("no config found.");
            eprintln!("please choose one of the following locations to create one:");

            for i in 0..config_paths.len() {
                eprintln!("{}: {}", i, config_paths[i]);
            }

            loop {
                eprint!("> ");

                io::stderr().flush().expect("failed to flush stdout");

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
            copy_default_config(config_path.as_ref());
            load_config_path(&mut s, &config_path.as_ref());
        }

        match s.try_into() {
            Ok(settings) => settings,
            Err(error) => {
                eprintln!("configuration error:\n{:#?}", error);
                std::process::exit(1);
            }
        }
    }
}
