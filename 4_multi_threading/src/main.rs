mod csv_mod;
use crate::csv_mod::csv_mod::read_csv;
use heck::AsSnakeCase;
use heck::AsUpperCamelCase;
use slug::slugify;
use std::env;
use std::error::Error;
use std::fmt;
use std::io;
use std::io::BufRead;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
enum Command {
    LowerCase,
    UpperCase,
    NoSpace,
    Slugify,
    CamelCase,
    SnakeCase,
    Csv,
    Unknown,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(input: &str) -> Result<Command, Self::Err> {
        match input {
            "lowercase" => Ok(Command::LowerCase),
            "uppercase" => Ok(Command::UpperCase),
            "nospace" => Ok(Command::NoSpace),
            "slugify" => Ok(Command::Slugify),
            "camelcase" => Ok(Command::CamelCase),
            "snakecase" => Ok(Command::SnakeCase),
            "csv" => Ok(Command::Csv),
            _ => Ok(Command::Unknown),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Command::LowerCase => write!(f, "lowercase"),
            Command::UpperCase => write!(f, "uppercase"),
            Command::NoSpace => write!(f, "nospace"),
            Command::Slugify => write!(f, "slugify"),
            Command::CamelCase => write!(f, "camelcase"),
            Command::SnakeCase => write!(f, "snakecase"),
            Command::Csv => write!(f, "csv"),
            Command::Unknown => write!(f, "unknown"),
        }
    }
}

// Structure for Command and argument
#[derive(Debug)]
struct ParsedCommand {
    command: Command,
    argument: String,
}

fn main() {
    //Create channel for data transfer
    let (tx, rx) = mpsc::channel();

    // thread for receiving input
    let input_thread = thread::spawn(move || {
        let arg = env::args().nth(1);

        match arg {
            Some(argument) => {
                match argument.parse::<Command>() {
                    Ok(command) => {
                        //read input from std
                        let mut text = String::new();
                        println!("Enter the text or csv file you want to convert:");
                        io::stdin()
                            .read_line(&mut text)
                            .expect("Failed to read line");
                        let parsed_command = ParsedCommand {
                            command,
                            argument: text,
                        };
                        if let Err(err) = tx.send(parsed_command) {
                            eprintln!("Error sending parsed command: {}", err);
                        }
                    }
                    Err(_) => {
                        eprintln!("Invalid input argument: {}", argument);
                    }
                }
            }
            None => {
                let stdin = io::stdin();
                let reader = stdin.lock();

                for line in reader.lines() {
                    match line {
                        Ok(input) => {
                            // Divide input on Command and argument
                            let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
                            if parts.len() == 2 {
                                let command_str = parts[0];
                                let argument = parts[1].to_string();

                                //Parse command using FromStr
                                match command_str.parse::<Command>() {
                                    Ok(command) => {
                                        let parsed_command = ParsedCommand { command, argument };
                                        if let Err(err) = tx.send(parsed_command) {
                                            eprintln!("Error sending parsed command: {}", err);
                                            break;
                                        }
                                    }
                                    Err(_) => {
                                        eprintln!("Invalid command: {}", command_str);
                                    }
                                }
                            } else {
                                eprintln!("Invalid input format. Expected: <command> <input>");
                            }
                        }
                        Err(error) => eprintln!("Error reading line: {}", error),
                    }
                }
            }
        }
    });

    // Thread for processing

    let processing_thread = thread::spawn(move || {
        while let Ok(parsed_command) = rx.recv() {
            let outtext = match parsed_command.command {
                Command::LowerCase => lowercase(
                    &parsed_command.command.to_string(),
                    &parsed_command.argument,
                ),
                Command::UpperCase => uppercase(
                    &parsed_command.command.to_string(),
                    &parsed_command.argument,
                ),
                Command::NoSpace => no_space(
                    &parsed_command.command.to_string(),
                    &parsed_command.argument,
                ),
                Command::Slugify => my_slugify(
                    &parsed_command.command.to_string(),
                    &parsed_command.argument,
                ),
                Command::CamelCase => camelcase(
                    &parsed_command.command.to_string(),
                    &parsed_command.argument,
                ),
                Command::SnakeCase => snakecase(
                    &parsed_command.command.to_string(),
                    &parsed_command.argument,
                ),
                Command::Csv => {
                    println!("bla {}", parsed_command.argument);
                    csv(
                        &parsed_command.command.to_string(),
                        &parsed_command.argument,
                    )
                }
                Command::Unknown => Err(Box::from(format!("Unknown command"))),
            };

            match outtext {
                Err(result) => {
                    eprintln!("{}", result)
                }
                Ok(result) => {
                    println!("{}", result)
                }
            }
        }
    });

    input_thread.join().unwrap(); // wait for threads to finish
    processing_thread.join().unwrap();
}

fn lowercase(argm: &String, text: &String) -> Result<String, Box<dyn Error>> {
    if argm != "lowercase" {
        Err(Box::from(format!(
            "lowercase function was called, meanwile argument is {}",
            argm
        )))
    } else {
        Ok(text.to_lowercase())
    }
}

fn uppercase(argm: &String, text: &String) -> Result<String, Box<dyn Error>> {
    if argm != "uppercase" {
        Err(Box::from(format!(
            "uppercase function was called, meanwile argument is {}",
            argm
        )))
    } else {
        Ok(text.to_uppercase())
    }
}

fn no_space(argm: &String, text: &String) -> Result<String, Box<dyn Error>> {
    if argm != "no-space" {
        Err(Box::from(format!(
            "no-space function was called, meanwile argument is {}",
            argm
        )))
    } else {
        Ok(text.replace(" ", ""))
    }
}

fn my_slugify(argm: &String, text: &String) -> Result<String, Box<dyn Error>> {
    if argm != "slugify" {
        Err(Box::from(format!(
            "slugify function was called, meanwile argument is {}",
            argm
        )))
    } else {
        Ok(slugify(text))
    }
}

fn camelcase(argm: &String, text: &String) -> Result<String, Box<dyn Error>> {
    if argm != "camelcase" {
        Err(Box::from(format!(
            "camelcase function was called, meanwile argument is {}",
            argm
        )))
    } else {
        Ok(AsUpperCamelCase(text).to_string())
    }
}

fn snakecase(argm: &String, text: &String) -> Result<String, Box<dyn Error>> {
    if argm != "snakecase" {
        Err(Box::from(format!(
            "snakecase function was called, meanwile argument is {}",
            argm
        )))
    } else {
        Ok(AsSnakeCase(text).to_string())
    }
}

fn csv(argm: &String, filename: &String) -> Result<String, Box<dyn Error>> {
    if argm != "csv" {
        Err(Box::from(format!(
            "csv function was called, meanwile argument is {}",
            argm
        )))
    } else {
        let rdr = read_csv(filename)?;
        Ok(rdr.to_string())
    }
}
