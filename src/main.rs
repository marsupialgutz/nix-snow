mod modes;

use std::{
    env::{ args, var },
    process::{ exit, Command, Stdio },
    io::Write,
    fs::read_to_string,
    str::from_utf8,
};
use modes::{ add::add_package, remove::remove_package };

fn main() {
    let args = args().collect::<Vec<String>>();

    match args.len() {
        1 => {
            eprintln!("Usage: nospm [add/remove] <package_name>");
            exit(1);
        }
        2 => {
            eprintln!("Please enter a package name.");
            exit(1);
        }
        3 => {
            let mut output_name = String::new();
            let mut output_new = Vec::new();

            let command = Command::new("nix")
                .args(&["search", "--json", "nixpkgs", &args[2]])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();

            let binding = command.wait_with_output().expect("Failed to wait on sed");
            let output = String::from_utf8_lossy(binding.stdout.as_slice());
            let parsed = json::parse(&output).unwrap();

            let mut packages = Vec::new();
            for (key, _) in parsed.entries() {
                packages.push(key.replacen("legacyPackages.x86_64-linux.", "", 1));
            }

            if packages.len() == 1 {
                output_name = String::from_utf8_lossy(packages[0].as_bytes()).to_string();
            }

            let file = temp_file::with_contents(output.as_bytes());

            if packages.len() > 1 {
                let mut fzf = Command::new("fzf")
                    .args(
                        &[
                            "--preview-window=wrap:45",
                            "--preview",
                            format!(
                                r#"cat {} | jq -rcs '.[0]["legacyPackages.x86_64-linux.{{}}"]["description"]'"#,
                                file.path().display()
                            ).as_str(),
                        ]
                    )
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap_or_else(|e| panic!("Failed to start fzf, error: {e}"));
                let stdin = fzf.stdin.as_mut().unwrap();

                stdin.write_all(packages.join("\n").as_bytes()).unwrap();

                output_new = fzf.wait_with_output().expect("Failed to wait on fzf").stdout;

                if output_new.is_empty() {
                    eprintln!("No package selected. Exiting...");
                    exit(1);
                }
            }

            let home_file: Vec<String> = read_to_string(
                format!("{}/nix-config/home/default.nix", var("HOME").unwrap())
            )
                .unwrap()
                .split('\n')
                .map(|x| x.to_string())
                .collect();

            let beginning = home_file
                .iter()
                .position(|x| x.contains("# NIX-ADD BEGIN"))
                .unwrap();

            let end = home_file
                .iter()
                .position(|x| x.contains("# NIX-ADD END"))
                .unwrap();

            let output_as_string = from_utf8(output_name.as_bytes()).unwrap().to_owned();
            let output_new_as_string = from_utf8(&output_new).unwrap().to_owned();

            match args[1].as_str() {
                "--help" => {
                    eprintln!("Usage: nospm [add/remove] <package_name>");
                    exit(0);
                }
                "add" => {
                    add_package(
                        home_file,
                        beginning,
                        end,
                        packages,
                        output_as_string,
                        output_new_as_string
                    );
                }
                "remove" => {
                    remove_package(
                        home_file,
                        beginning,
                        end,
                        packages,
                        output_as_string,
                        output_new_as_string
                    );
                }
                _ => {
                    eprintln!("Please enter a valid command.");
                    exit(1);
                }
            }
        }
        _ => {
            eprintln!("Too many arguments.");
            exit(1);
        }
    }
}
