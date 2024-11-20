extern crate argparse;

use std::io::{stderr, stdout};
use std::str::FromStr;
use argparse::{ArgumentParser, StoreTrue, Store, List};
use std::env;

#[derive(Debug)]
enum Command {
    Download,
    Upload,
}

impl FromStr for Command {
    type Err = ();
    fn from_str(src: &str) -> Result<Command, ()> {
        return match src {
            "download" => Ok(Command::Download),
            "dl" => Ok(Command::Download),
            "upload" => Ok(Command::Upload),
            "up" => Ok(Command::Upload),
            _ => Err(()),
        };
    }
}

fn download(verbose:bool, args: Vec<String>) {
    let mut view_name = String::new();
    let mut filter_string = String::new();
    let mut file_path = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("CoSelPro Download query dataset");
        ap.refer(&mut view_name).required().add_argument("query", Store, "Query name");
        ap.refer(&mut filter_string).add_option(&["-f", "--filter"], Store, "Query filter");
        ap.refer(&mut file_path).add_option(&["-o", "--output"], Store, "Output file path");

        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(err) => {
                std::process::exit(err);
            }
        }
    }
    print!("Download dataset from {:?}", &view_name);
    if !filter_string.is_empty() {print!(" with filter = {:?}", filter_string);}
    let output_path:String = match file_path.is_empty() {
        false => {file_path},
        true => { match env::current_exe() {
            Ok(path) => {
                let mut pth = path.into_os_string().into_string().unwrap();
                pth.push_str(".xlsx");
                pth
            },
            _ => {"coselpro.report.xlsx".to_string()}
        }
        }
    };
    print!(" to {:?}", &output_path);
    if verbose {print!(" in verbose mode")};
    println!(".");
}

fn upload(verbose:bool, args: Vec<String>) {
    print!("Upload dataset {:?}", args);
    if verbose {print!(" with verbose mode")};
    println!(".");
}
fn main() {
    let mut verbose = false;
    let mut command:Command = Command::Download;
    let mut args:Vec<String> = Vec::new();
    {  // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("COmponent SELection & PROcurement.");
        ap.refer(&mut verbose).add_option(&["-v", "--verbose"], StoreTrue, "Verbose");
        ap.refer(&mut command).required().add_argument("command", Store, "CoSelPro Command");
        ap.refer(&mut args).add_argument("arguments", List, "Command Arguments");
        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }

    args.insert(0,format!("command {:?}", command));
    match command {
        Command::Download => download(verbose, args),
        Command::Upload => upload(verbose, args),
    }
}