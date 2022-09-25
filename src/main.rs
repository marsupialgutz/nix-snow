use spinoff::{Color, Spinner, Spinners};

mod modes;
use {
    bpaf::Bpaf,
    json::parse,
    modes::{add::add_package, remove::remove_package},
    once_cell::sync::Lazy,
    serde_derive::Deserialize,
    std::{
        env::var,
        fs::read_to_string,
        io::Write,
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
#[bpaf(options, version)]
struct Args {
    #[bpaf(short('d'))]
    dry_run: bool,
    #[bpaf(short('a'))]
    add: Option<String>,
    #[bpaf(short('r'))]
    remove: Option<String>,
}

fn main() {
    let opts = args().run();

    if opts.add.is_none() && opts.remove.is_none() {
        eprintln!(r#"You must either add or remove a package. Use "-h" or "--help" for usage."#);
        exit(1);
    } else if opts.add.is_some() && opts.remove.is_some() {
        eprintln!(
            r#"You can only add or remove a package, not both. Use "-h" or "--help" for usage."#
        );
        exit(1);
    }

    let output_str;

    let sp = Spinner::new(
        Spinners::Dots,
        format!("Searching for {}", get_pkg(&opts)),
        Color::Blue,
    );

    let cmd = Command::new("nix")
        .args(&["search", "--json", "nixpkgs", &get_pkg(&opts)])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("Failed to run nix search: {e}");
            exit(1);
        });

    let binding = cmd.wait_with_output().unwrap();
    let out = String::from_utf8_lossy(binding.stdout.as_slice());
    let parsed = parse(&out).unwrap();

    let mut pkgs = Vec::new();
    for (key, _) in parsed.entries() {
        pkgs.push(key.replace("legacyPackages.x86_64-linux.", ""))
    }

    sp.success("Done!");

    if pkgs.is_empty() {
        eprintln!("Package not found: {}", get_pkg(&opts));
        exit(1);
    } else if pkgs.len() == 1 {
        output_str = String::from_utf8_lossy(pkgs[0].as_bytes()).to_string();
    } else {
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
                eprintln!("Failed to start fzf: {e}");
                exit(1);
            });
        let stdin = search.stdin.as_mut().unwrap();

        stdin
            .write_all(pkgs.join("\n").as_bytes())
            .unwrap_or_else(|e| {
                eprintln!("Failed to list packages: {e}");
                exit(1);
            });

        let res = search
            .wait_with_output()
            .unwrap_or_else(|e| {
                eprintln!("Failed to wait on fzf: {e}");
                exit(1);
            })
            .stdout;

        if res.is_empty() {
            eprintln!("No package selected");
            exit(1);
        }

        output_str = from_utf8(&res).unwrap().trim_end().to_string();
    }

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

    if opts.add.is_some() && opts.remove.is_none() {
        add_package(file, output_str);
    } else if opts.remove.is_some() && opts.add.is_none() {
        remove_package(file, output_str);
    }
}

fn read_config() -> Config {
    let content = read_to_string(format!(
        "{}/.config/nix-snow/config.toml",
        var("HOME").unwrap_or_else(|e| {
            eprintln!("Failed to read config file: {e}");
            exit(1);
        })
    ))
    .unwrap();
    from_str(&content).unwrap_or_else(|e| {
        eprintln!("Cannot find config file: {e}");
        exit(1);
    })
}

fn get_pkg(opts: &Args) -> String {
    if let Some(add) = &opts.add {
        add.into()
    } else if let Some(remove) = &opts.remove {
        remove.into()
    } else {
        eprintln!("Package was not specified");
        exit(1);
    }
}
