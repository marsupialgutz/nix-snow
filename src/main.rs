mod modes;
use {
    bpaf::Bpaf,
    json::parse,
    modes::{add::add_package, remove::remove_package},
    once_cell::sync::Lazy,
    serde_derive::Deserialize,
    spinoff::{Color, Spinner, Spinners},
    std::{
        env::{set_current_dir, var},
        fs::read_to_string,
        io::{stdin, stdout, Write},
        process::{exit, Command, Stdio},
        str::from_utf8,
    },
    temp_file::with_contents,
    toml::from_str,
};

#[derive(Deserialize)]
pub struct Config {
    rebuild: String,
    path: Option<String>,
}

pub static CONFIG: Lazy<Config> = Lazy::new(read_config);

#[derive(Clone, Debug, Bpaf)]
enum Action {
    Add(
        /// Add a package
        #[bpaf(long("add"), short('a'), argument("PACKAGE"))]
        String,
    ),
    Remove(
        /// Remove a package
        #[bpaf(long("remove"), short('r'), argument("PACKAGE"))]
        String,
    ),
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)]
/// Nix-snow - add packages to your nix configuration
struct Args {
    /// Custom config file to use
    #[bpaf(long, short, argument("FILE"))]
    config: Option<String>,
    /// Dry-run, don't change files
    #[bpaf(long, short)]
    dry_run: bool,
    #[bpaf(external)]
    action: Action,
    /// Don't rebuild if you have "always rebuild" on
    #[bpaf(long, short)]
    no_rebuild: bool,
}

pub fn run_rebuild() {
    match CONFIG.rebuild.as_str() {
        "always" => {
            let sp = Spinner::new(Spinners::Dots, "Rebuilding...", Color::Blue);
            set_current_dir(format!("{}/nix-config", var("HOME").unwrap())).unwrap();
            Command::new(format!("{}/nix-config/bin/build", var("HOME").unwrap()))
                .arg("-h")
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
            sp.success("Successfully rebuilt!");
        }
        "ask" => {
            print!("Would you like to rebuild now? (y/n): ");
            let mut response = String::new();
            stdout().flush().unwrap();
            stdin().read_line(&mut response).unwrap();

            if response.trim() == "y" {
                let sp = Spinner::new(Spinners::Dots, "Rebuilding...", Color::Blue);
                set_current_dir(format!("{}/nix-config", var("HOME").unwrap())).unwrap();
                Command::new(format!("{}/nix-config/bin/build", var("HOME").unwrap()))
                    .arg("-h")
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
                sp.success("Successfully rebuilt!");
            }
        }
        "never" => (),
        _ => {
            eprintln!("\x1b[31m✗\x1b[0m Unknown setting");
            exit(1);
        }
    }
}

fn main() {
    let opts = args().run();

    let rebuild = !opts.no_rebuild;

    let output_str = get_name(&opts);

    if opts.dry_run {
        exit(0)
    }

    let file = read_to_string({
        if let Some(path) = &CONFIG.path {
            path.replace('~', &var("HOME").unwrap())
        } else {
            format!("{}/nix-config/home/default.nix", var("HOME").unwrap())
        }
    })
    .unwrap()
    .split('\n')
    .map(|x| x.to_string())
    .collect::<Vec<_>>();

    match opts.action {
        Action::Add(..) => {
            add_package(file, output_str, rebuild);
        }
        Action::Remove(..) => {
            remove_package(file, output_str, rebuild);
        }
    }
}

fn read_config() -> Config {
    let opts = args().run();

    let content = read_to_string(if let Some(path) = opts.config {
        path.replace('~', &var("HOME").unwrap())
    } else {
        format!("{}/.config/nix-snow/config.toml", var("HOME").unwrap())
    })
    .unwrap_or_else(|e| {
        eprintln!("\x1b[31m✗\x1b[0m Cannot find config file: {e}");
        exit(1);
    });
    from_str(&content).unwrap()
}

fn get_pkg(opts: &Args) -> String {
    match &opts.action {
        Action::Add(name) | Action::Remove(name) => name.to_owned(),
    }
}

fn get_name(opts: &Args) -> String {
    let sp = Spinner::new(
        Spinners::Dots,
        format!("Searching for {}...", get_pkg(opts)),
        Color::Blue,
    );

    let cmd = Command::new("nix")
        .args(&["search", "--json", "nixpkgs", &get_pkg(opts)])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("\x1b[31m✗\x1b[0m Failed to run nix search: {e}");
            exit(1);
        });

    let binding = cmd.wait_with_output().unwrap();
    let out = String::from_utf8_lossy(binding.stdout.as_slice());
    let parsed = parse(&out).unwrap();

    let mut pkgs = Vec::new();
    for (key, _) in parsed.entries() {
        pkgs.push(key.replace("legacyPackages.x86_64-linux.", ""))
    }

    if pkgs.is_empty() {
        sp.fail(&format!("Package not found: {}", get_pkg(opts)));
        exit(1);
    } else if pkgs.len() == 1 {
        sp.success(&format!("Found {}!", get_pkg(opts)));
        String::from_utf8_lossy(pkgs[0].as_bytes()).to_string()
    } else {
        sp.success(&format!("Found {}!", get_pkg(opts)));
        let temp_file = with_contents(out.as_bytes());
        let mut search = Command::new("fzf")
            .args(&[
                "--preview-window=wrap:45",
                "--preview",
                format!(
                    r#"cat {} | jq -rcs '.[0]["legacyPackages.x86_64-linux.{{}}"]["description"]'"#,
                    temp_file.path().display()
                )
                .as_str(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| {
                eprintln!("\x1b[31m✗\x1b[0m Failed to start fzf: {e}");
                exit(1);
            });
        let stdin = search.stdin.as_mut().unwrap();

        stdin
            .write_all(pkgs.join("\n").as_bytes())
            .unwrap_or_else(|e| {
                eprintln!("\x1b[31m✗\x1b[0m Failed to list packages: {e}");
                exit(1);
            });

        let res = search
            .wait_with_output()
            .unwrap_or_else(|e| {
                eprintln!("\x1b[31m✗\x1b[0m Failed to wait on fzf: {e}");
                exit(1);
            })
            .stdout;

        if res.is_empty() {
            eprintln!("\x1b[31m✗\x1b[0m No package selected");
            exit(1);
        }

        from_utf8(&res).unwrap().trim_end().to_string()
    }
}
