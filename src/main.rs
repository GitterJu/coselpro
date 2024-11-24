extern crate argparse;
extern crate log4rs;
use log::{debug, error, info, warn};

use argparse::{ArgumentParser, List, Store, StoreTrue};
use std::env;
use std::io::{stderr, stdout};
use std::str::FromStr;

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

fn download(verbose: bool, args: Vec<String>) {
    debug!("Downloading function. Extracting arguments");
    let mut view_name = String::new();
    let mut filter_string = String::new();
    let mut file_path = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("CoSelPro Download query dataset");
        ap.refer(&mut view_name)
            .required()
            .add_argument("query", Store, "Query name");
        ap.refer(&mut filter_string)
            .add_option(&["-f", "--filter"], Store, "Query filter");
        ap.refer(&mut file_path)
            .add_option(&["-o", "--output"], Store, "Output file path");

        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(err) => {
                std::process::exit(err);
            }
        }
    }

    let mut msg = format!("Download dataset from {:?}", &view_name);
    if !filter_string.is_empty() {
        msg.push_str(&format!(" with filter = {:?}", filter_string));
    }
    let output_path: String = match file_path.is_empty() {
        false => file_path,
        true => match env::current_exe() {
            Ok(path) => {
                let mut pth = path.into_os_string().into_string().unwrap();
                pth.push_str(".xlsx");
                pth
            }
            _ => "coselpro.report.xlsx".to_string(),
        },
    };
    msg.push_str(&format!(" to {:?}", &output_path));
    if verbose {
        msg.push_str(" in verbose mode")
    };
    msg.push_str(".");
    info!("{}", &msg);
}

fn upload(verbose: bool, args: Vec<String>) {
    print!("Upload dataset {:?}", args);
    if verbose {
        print!(" with verbose mode")
    };
    println!(".");
}

mod api;
fn main() {
    log4rs::init_file("logging.yml", Default::default()).unwrap();
    info!("Starting CoSelPro…");

    use api::connection::prompt_console_credentials;
    println!("{:?}", prompt_console_credentials().unwrap());

    debug!("Extracting arguments…");
    let mut verbose = false;
    let mut command: Command = Command::Download;
    let mut args: Vec<String> = Vec::new();
    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("COmponent SELection & PROcurement.");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Verbose");
        ap.refer(&mut command)
            .required()
            .add_argument("command", Store, "CoSelPro Command");
        ap.refer(&mut args)
            .add_argument("arguments", List, "Command Arguments");
        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }

    args.insert(0, format!("command {:?}", command));
    match command {
        Command::Download => download(verbose, args),
        Command::Upload => upload(verbose, args),
    }
}
