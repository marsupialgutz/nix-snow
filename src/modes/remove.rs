use {
    crate::CONFIG,
    std::{
        env::{set_current_dir, var},
        fs::write,
        io::{stdin, stdout, Write},
        process::{exit, Command},
    },
};

pub fn remove_package(file: Vec<String>, package: String) {
    if let Some(beginning) = file.iter().position(|x| x.trim().contains("# SNOW BEGIN")) {
        if let Some(end) = file.iter().position(|x| x.trim().contains("# SNOW END")) {
            let new_file = file
                .iter()
                .enumerate()
                .filter(|(i, x)| !(*i >= beginning && *i <= end && x.trim().eq(&package)))
                .map(|(_, x)| x.to_string())
                .collect::<Vec<_>>();

            if new_file == file {
                eprintln!("Package {package} is not in your list of Nix packages.");
                exit(1);
            }

            write(
                {
                    if let Some(path) = &CONFIG.path {
                        path.replace('~', &var("HOME").unwrap())
                    } else {
                        format!("{}/nix-config/home/default.nix", var("HOME").unwrap())
                    }
                },
                new_file.join("\n"),
            )
            .unwrap();
        }
    }

    println!("Removed {package} from your Nix packages.");

    match CONFIG.rebuild.as_str() {
        "always" => {
            set_current_dir(format!("{}/nix-config", var("HOME").unwrap())).unwrap();
            Command::new(format!("{}/nix-config/bin/build", var("HOME").unwrap()))
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }
        "ask" => {
            print!("Would you like to rebuild now? (Y/n): ");
            let mut response = String::new();
            stdout().flush().unwrap();
            stdin().read_line(&mut response).unwrap();

            if response.trim() == "y" || response.trim() == "" {
                set_current_dir(format!("{}/nix-config", var("HOME").unwrap())).unwrap();
                Command::new(format!("{}/nix-config/bin/build", var("HOME").unwrap()))
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
            }
        }
        "never" => (),
        _ => panic!("Unknown setting {}", CONFIG.rebuild),
    }
}
