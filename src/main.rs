mod modes;
use {
    bpaf::Bpaf,
    json::parse,
    modes::{add::add_package, remove::remove_package},
    once_cell::sync::Lazy,
    serde_derive::Deserialize,
    spinoff::{Color, Spinner, Spinners},
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

    if opts.dry_run {
        println!("dry run");
    }

    if let Some(add) = opts.add {
        println!("Adding {add}");
    } else if let Some(remove) = opts.remove {
        println!("Removing {remove}");
    }
}

fn read_config() -> Config {
    let content = read_to_string({
        if let Some(p) = &CONFIG.path {
            p.replace("~", &var("HOME").unwrap()).to_owned()
        } else {
            format!("{}/nix-config/home/default.nix", var("HOME").unwrap())
        }
    })
    .unwrap();
    from_str(&content).unwrap()
}
