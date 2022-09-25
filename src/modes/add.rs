use {
    crate::CONFIG,
    std::{
        env::{set_current_dir, var},
        fs::write,
        io::{stdin, stdout, Write},
        process::{exit, Command},
    },
};

pub fn add_package(mut file: Vec<String>, package: String) {
    if let Some(beginning) = file.iter().position(|x| x.trim().contains("# SNOW BEGIN")) {
        if let Some(end) = file.iter().position(|x| x.trim().contains("# SNOW END")) {
            let whitespace = file[beginning]
                .chars()
                .take_while(|x| x.is_whitespace())
                .collect::<String>();

            if file[beginning..end]
                .iter()
                .any(|x| x.trim() == package.trim())
            {
                eprintln!("Package already installed, not adding.");
                exit(1);
            }

            file.insert(beginning + 1, whitespace + &package);
            file[beginning..end].sort();
        }
    }

    write(
        {
            if let Some(path) = &CONFIG.path {
                path.replace('~', &var("HOME").unwrap())
            } else {
                format!("{}/nix-config/home/default.nix", var("HOME").unwrap())
            }
        },
        file.join("\n"),
    )
    .unwrap();

    println!("Added {package} to your Nix packages.");

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
            print!("Would you like to rebuild now? (y/n): ");
            let mut response = String::new();
            stdout().flush().unwrap();
            stdin().read_line(&mut response).unwrap();

            if response.trim() == "y" {
                set_current_dir(format!("{}/nix-config", var("HOME").unwrap())).unwrap();
                Command::new(format!("{}/nix-config/bin/build", var("HOME").unwrap()))
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
            }
        }
        "never" => (),
        _ => panic!("Unknown setting"),
    }
}
