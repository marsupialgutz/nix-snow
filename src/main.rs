mod modes;

use std::{ env::args, process::exit };
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
            match args[1].as_str() {
                "--help" => {
                    eprintln!("Usage: nospm [add/remove] <package_name>");
                    exit(0);
                }
                "add" => {
                    add_package(&args[2]);
                }
                "remove" => {
                    remove_package(&args[2]);
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
