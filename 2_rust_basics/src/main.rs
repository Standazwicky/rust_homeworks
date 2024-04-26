use heck::AsSnakeCase;
use heck::AsUpperCamelCase;
use slug::slugify;
use std::env;
use std::io;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        help();
    } else {
        println!("Enter the text you want to convert:");

        match args[1].as_str() {
            "lowercase" => {
                let mut text = String::new();
                io::stdin()
                    .read_line(&mut text)
                    .expect("Failed to read line");
                println!("{}", text.to_lowercase());
            }
            "uppercase" => {
                let mut text = String::new();
                io::stdin()
                    .read_line(&mut text)
                    .expect("Failed to read line");
                println!("{}", text.to_uppercase());
            }
            "no-space" => {
                let mut text = String::new();
                io::stdin()
                    .read_line(&mut text)
                    .expect("Failed to read line");
                println!("{}", text.replace(" ", ""));
            }
            "slugify" => {
                let mut text = String::new();
                io::stdin()
                    .read_line(&mut text)
                    .expect("Failed to read line");
                println!("{}", slugify(text));
            }
            "camelcase" => {
                let mut text = String::new();
                io::stdin()
                    .read_line(&mut text)
                    .expect("Failed to read line");
                println!("{}", AsUpperCamelCase(text));
            }
            "snakecase" => {
                let mut text = String::new();
                io::stdin()
                    .read_line(&mut text)
                    .expect("Failed to read line");
                println!("{}", AsSnakeCase(text));
            }
            _ => {
                println! {"Wrong input argument."};
                println! {};
                help();
            }
        }
    }
}

fn help() {
    println!("Use one input argument.");
    println!("");
    println!("Options:");
    println!("lowercase - convert the entire text to lowercase");
    println!("uppercase - convert the entire text to uppercase");
    println!("no-spaces - remove all spaces from the text");
    println!("slugify - convert the text into a slug");
    println!("camelcase - convert the text into a UpperCamelCase");
    println!("snakecase - convert the text into a snake_case");
}
