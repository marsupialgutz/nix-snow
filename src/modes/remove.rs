use {
  colorful::Colorful,
  crate::{CONFIG, run_rebuild},
  std::{env::var, fs::write, process::exit},
};

pub fn remove_package(file: Vec<String>, package: String, rebuild: bool) {
  if let Some(beginning) = file
    .iter()
    .position(|x| x.trim().contains("# SNOW BEGIN"))
    && let Some(end) = file
    .iter()
    .position(|x| x.trim().contains("# SNOW END"))
  {
    let new_file = file
      .iter()
      .enumerate()
      .filter(
        |(i, x)| !(*i >= beginning && *i <= end && x.trim().eq(&package))
      )
      .map(|(_, x)| x.to_string())
      .collect::<Vec<_>>();

    if new_file == file {
      eprintln!(
        "{} Package {package} is not in your list of Nix packages.",
        "✗".red()
      );
      exit(1);
    }

    write(
      {
        CONFIG.get("path").map_or_else(
          || format!("{}/nix-config/home/default.nix", var("HOME").unwrap()),
          |path| path.replace('~', &var("HOME").unwrap()),
        )
      },
      new_file.join("\n"),
    ).unwrap();
  }

  println!("{} Removed {package} from your Nix packages.", "✓".green());

  if rebuild {
    run_rebuild();
  }
}