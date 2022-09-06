use {
    crate::CONFIG,
    std::{
        env::{set_current_dir, var},
        fs::write,
        io::{stdin, stdout, Write},
        process::{exit, Command},
    },
};

pub fn remove_package(
    home_file: Vec<String>,
    beginning: usize,
    end: usize,
    packages: Vec<String>,
    output: String,
    output_new: String,
) {
    let home_file_new = home_file
        .iter()
        .enumerate()
        .filter(|(i, x)| {
            !(*i >= beginning
                && *i <= end
                && x.trim().eq({
                    {
                        if packages.len() > 1 {
                            output_new.trim()
                        } else {
                            output.trim()
                        }
                    }
                }))
        })
        .map(|(_, x)| x.to_string())
        .collect::<Vec<_>>();

    if home_file_new == home_file {
        eprintln!("Package {} is not in your list of Nix packages.", {
            if packages.len() > 1 {
                output_new.trim()
            } else {
                output.trim()
            }
        });
        exit(1);
    }

    write(
        format!("{}/nix-config/home/default.nix", var("HOME").unwrap()),
        home_file_new.join("\n"),
    )
    .unwrap();

    println!("Removed {} from your Nix packages.", {
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
