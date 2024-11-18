extern crate argparse;

use std::io::{stderr, stdout};
use std::str::FromStr;
use argparse::{ArgumentParser, StoreTrue, Store, List};

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
    let mut sargs:Vec<String> = Vec::new();
    let mut file_path = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("CoSelPro Download query dataset");
        ap.refer(&mut view_name).required().add_argument("query", Store, "Query name");
        ap.refer(&mut sargs).add_argument("arguments", List, "Query arguments");
        ap.refer(&mut file_path).add_option(&["-f", "--file"], Store, "Path to file");

        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {
                println!("Download dataset {:?}", &view_name)
            }
            Err(err) => {
                std::process::exit(err);
            }
        }
    }
}

fn upload(verbose:bool, args: Vec<String>) {

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