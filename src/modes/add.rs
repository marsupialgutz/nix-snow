use std::{
    env::{set_current_dir, var},
    fs::write,
    io::{stdin, stdout, Write},
    process::{exit, Command},
};

use crate::CONFIG;

pub fn add_package(
    mut home_file: Vec<String>,
    beginning: usize,
    end: usize,
    packages: Vec<String>,
    output: String,
    output_new: String,
) {
    let whitespace = home_file[beginning]
        .chars()
        .take_while(|x| x.is_whitespace())
        .collect::<String>();

    if home_file[beginning..end]
        .iter()
        .any(|x| *x.trim() == *output_new.trim())
        || (packages.len() <= 1
            && home_file[beginning..end]
                .iter()
                .any(|x| *x.trim() == *output.trim()))
    {
        eprintln!("Package already installed, not adding.");
        exit(1);
    }

    home_file.insert(
        beginning + 1,
        whitespace
            + ({
                if packages.len() > 1 {
                    output_new.trim()
                } else {
                    output.trim()
                }
            }),
    );

    home_file[beginning..end].sort();

    write(
        format!("{}/nix-config/home/default.nix", var("HOME").unwrap()),
        home_file.join("\n"),
    )
    .unwrap();

    println!("Added {} to your Nix packages.", {
        if packages.len() > 1 {
            output_new.trim()
        } else {
            output.trim()
        }
    });

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
