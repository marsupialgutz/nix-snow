use std::{
    process::{ Command, Stdio, exit },
    io::{ Write, stdout, stdin },
    fs::{ read_to_string, write },
    env::{ var, set_current_dir },
    str::from_utf8,
};

pub fn add_package(package_name: &String) {
    let mut output_name = String::new();
    let mut output_new = Vec::new();

    let command = Command::new("nix")
        .args(&["search", "--json", "nixpkgs", package_name])
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

    let mut home_file: Vec<String> = read_to_string(
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

    let whitespace = home_file[beginning]
        .chars()
        .take_while(|x| x.is_whitespace())
        .collect::<String>();

    let output_as_string = from_utf8(output_name.as_bytes()).unwrap().to_owned();
    let output_new_as_string = from_utf8(&output_new).unwrap().to_owned();

    if
        home_file[beginning..end].iter().any(|x| { *x.trim() == *output_new_as_string.trim() }) ||
        (packages.len() <= 1 &&
            home_file[beginning..end].iter().any(|x| *x.trim() == *output_as_string.trim()))
    {
        eprintln!("Package already installed, not adding.");
        exit(1);
    }

    home_file.insert(
        beginning + 1,
        whitespace +
            ({
                if packages.len() > 1 {
                    output_new_as_string.trim()
                } else {
                    output_as_string.trim()
                }
            })
    );

    home_file[beginning..end].sort();

    write(
        format!("{}/nix-config/home/default.nix", var("HOME").unwrap()),
        home_file.join("\n")
    ).unwrap();

    println!("Added {} to your Nix packages.", {
        if packages.len() > 1 { output_new_as_string.trim() } else { output_as_string.trim() }
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
