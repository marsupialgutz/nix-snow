use crate::rebuild;

use {
    crate::CONFIG,
    std::{env::var, fs::write, process::exit},
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
                eprintln!(
                    "\x1b[31m✗\x1b[0m Package {package} is not in your list of Nix packages."
                );
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

    println!("✓ Removed {package} from your Nix packages.");

    rebuild();
}
