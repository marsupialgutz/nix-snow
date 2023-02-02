#![feature(let_chains)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::needless_pass_by_value)]

use {
  clap::{Parser, Subcommand},
  colorful::Colorful,
  config::Config,
  config::File,
  json::parse,
  modes::{add::add_package, remove::remove_package},
  once_cell::sync::Lazy,
  spinoff::{Color, Spinner, spinners},
  std::{
    collections::HashMap,
    env::{set_current_dir, var},
    fs::read_to_string,
    io::{stdin, stdout, Write},
    process::{Command, exit, Stdio},
    str::from_utf8,
  },
  temp_file::with_contents,
};

mod modes;

pub static CONFIG: Lazy<HashMap<String, String>> = Lazy::new(||
  read_config().try_deserialize::<HashMap<String, String>>().unwrap()
);

pub static ARGS: Lazy<Args> = Lazy::new(Args::parse);

#[derive(Clone, Debug, Subcommand)]
pub enum Action {
  /// Add a package
  #[command(alias = "a")]
  Add {
    pkg: String,
  },
  /// Remove a package
  #[command(alias = "rm")]
  Remove {
    pkg: String,
  },
}

// Add/remove packages from your nix configuration
#[derive(Clone, Debug, Parser)]
#[clap(name = "nix-snow", version = "0.1.0", author = "pupbrained")]
pub struct Args {
  /// Custom config file to use
  #[arg(short, long)]
  config: Option<String>,
  /// Dry-run, don't change files
  #[arg(short, long)]
  dry_run: bool,
  #[command(subcommand)]
  action: Action,
  /// Don't rebuild if you have "always rebuild" on
  #[arg(short, long)]
  no_rebuild: bool,
}

pub fn run_rebuild() {
  if let Some(rebuild) = CONFIG.get("rebuild") {
    match rebuild.as_str() {
      "always" => {
        let sp = Spinner::new(spinners::Dots, "Rebuilding...", Color::Blue);
        set_current_dir(
          format!("{}/nix-config", var("HOME").unwrap())
        ).unwrap();
        let cmd = Command::new(
          format!("{}/nix-config/bin/build", var("HOME").unwrap())
        )
          .arg("h")
          .stdout(Stdio::null())
          .stderr(Stdio::piped())
          .status()
          .unwrap();
        if !cmd.success() {
          sp.fail("Failed to rebuild");
          exit(1);
        }
        sp.success("Successfully rebuilt!");
      }
      "ask" => {
        print!("Would you like to rebuild now? (y/n): ");
        let mut response = String::new();
        stdout().flush().unwrap();
        stdin().read_line(&mut response).unwrap();

        if response.trim() == "y" {
          let sp = Spinner::new(spinners::Dots, "Rebuilding...", Color::Blue);
          set_current_dir(
            format!("{}/nix-config", var("HOME").unwrap())
          ).unwrap();
          let cmd = Command::new(
            format!("{}/nix-config/bin/build", var("HOME").unwrap())
          )
            .arg("-h")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .status()
            .unwrap();
          if !cmd.success() {
            sp.fail("Failed to rebuild");
            exit(1);
          }
          sp.success("Successfully rebuilt!");
        }
      }
      "never" => (),
      _ => {
        eprintln!("{} Unknown setting", "✗".red());
        exit(1);
      }
    }
  }
}

fn main() {
  let rebuild = !ARGS.no_rebuild;

  let file = read_to_string({
    CONFIG.get("path").map_or_else(
      || format!("{}/nix-config/home/default.nix", var("HOME").unwrap()),
      |path| path.replace('~', &var("HOME").unwrap()),
    )
  })
    .unwrap()
    .split('\n')
    .map(std::string::ToString::to_string)
    .collect::<Vec<String>>();

  let output_str = get_name(&file);

  if ARGS.dry_run {
    exit(0);
  }

  match &ARGS.action {
    Action::Add { .. } => {
      add_package(file, output_str, rebuild);
    }
    Action::Remove { .. } => {
      remove_package(file, output_str, rebuild);
    }
  }
}

fn read_config() -> Config {
  Config::builder()
    .add_source(
      File::with_name(
        &ARGS.clone().config.map_or_else(|| format!("{}/.config/nix-snow/config.toml", var("HOME").unwrap()), |path| path.replace('~', &var("HOME").unwrap()))
      )
    )
    .build()
    .unwrap()
}

fn get_pkg() -> String {
  match &ARGS.action {
    Action::Add { pkg } | Action::Remove { pkg } => pkg.clone(),
  }
}

fn get_name(file: &[String]) -> String {
  if let Some(beginning) = file
    .iter()
    .position(|x| x.trim().contains("# SNOW BEGIN")) &&
    let Some(end) = file.iter().position(|x| x.trim().contains("# SNOW END")) &&
    let Action::Remove { pkg } = &ARGS.action
  {
    for i in file[beginning..end].iter() {
      if i.trim() == pkg.trim() {
        return i.trim().to_owned();
      }
    }
  }

  let sp = Spinner::new(
    spinners::Dots,
    format!("Searching for {}...", get_pkg()),
    Color::Blue,
  );

  let cmd = Command::new("nix")
    .args(["search", "--json", "nixpkgs", &get_pkg()])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap_or_else(|e| {
      eprintln!("{} Failed to run nix search: {e}", "✗".red());
      exit(1);
    });

  let binding = cmd.wait_with_output().unwrap();
  let out = String::from_utf8_lossy(binding.stdout.as_slice());
  let parsed = parse(&out).unwrap();

  let mut pkgs = Vec::new();
  for (key, _) in parsed.entries() {
    pkgs.push(key.replace("legacyPackages.x86_64-linux.", ""));
  }

  if pkgs.is_empty() {
    sp.fail(&format!("Package not found: {}", get_pkg()));
    print!("Do you want to add it anyway? (Y/n): ");
    let mut response = String::new();
    stdout().flush().unwrap();
    stdin().read_line(&mut response).unwrap();
    match response.trim().to_lowercase().as_str() {
      "y" | "" => get_pkg(),
      "n" => exit(0),
      _ => {
        eprintln!("{} Unknown response", "✗".red());
        exit(1);
      }
    }
  } else if pkgs.len() == 1 {
    sp.success(&format!("Found {}!", get_pkg()));
    String::from_utf8_lossy(pkgs[0].as_bytes()).to_string()
  } else {
    sp.success(&format!("Found {}!", get_pkg()));
    let temp_file = with_contents(out.as_bytes());
    let mut search = Command::new("fzf")
      .args([
        "--preview-window=wrap:45",
        "--preview",
        format!(
          r#"cat {} | jq -rcs '.[0]["legacyPackages.x86_64-linux.{{}}"]["description"]'"#,
          temp_file.path().display()
        ).as_str(),
      ])
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn()
      .unwrap_or_else(|e| {
        eprintln!("{} Failed to start fzf: {e}", "✗".red());
        exit(1);
      });
    let stdin = search.stdin.as_mut().unwrap();

    stdin.write_all(pkgs.join("\n").as_bytes()).unwrap_or_else(|e| {
      eprintln!("{} Failed to list packages: {e}", "✗".red());
      exit(1);
    });

    let res = search.wait_with_output().unwrap_or_else(|e| {
      eprintln!("{} Failed to wait on fzf: {e}", "✗".red());
      exit(1);
    }).stdout;

    if res.is_empty() {
      eprintln!("{} No package selected", "✗".red());
      exit(1);
    }

    from_utf8(&res).unwrap().trim_end().to_string()
  }
}