use std::{
    process::Command,
    io::{ Write, stdout, stdin },
    fs::write,
    env::{ var, set_current_dir },
};

pub fn remove_package(
    mut home_file: Vec<String>,
    beginning: usize,
    end: usize,
    packages: Vec<String>,
    output: String,
    output_new: String
) {
    home_file = home_file
        .iter()
        .enumerate()
        .filter(|(i, x)| {
            !(
                *i >= beginning &&
                *i <= end &&
                x.trim().eq({
                    {
                        if packages.len() > 1 { output_new.trim() } else { output.trim() }
                    }
                })
            )
        })
        .map(|(_, x)| x.to_string())
        .collect();

    write(
        format!("{}/nix-config/home/default.nix", var("HOME").unwrap()),
        home_file.join("\n")
    ).unwrap();

    println!("Removed {} from your Nix packages.", {
        if packages.len() > 1 { output_new.trim() } else { output.trim() }
    });

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
