use heck::AsSnakeCase;
use heck::AsUpperCamelCase;
use slug::slugify;
use std::env;
use std::io;
use std::error::Error;
//use std::{error::Error, io};
// mod csv_mod;

fn main() {
    let arg = env::args().nth(1);

    let outtext=match &arg {
         Some(argument) if argument == "lowercase" => lowercase(&argument),
         Some(argument) if argument == "uppercase" => uppercase(&argument),
         Some(argument) if argument == "no-space" => no_space(&argument),
         Some(argument) if argument == "slugify" => my_slugify(&argument),
         Some(argument) if argument == "camelcase" => camelcase(&argument),
         Some(argument) if argument == "snakecase" => snakecase(&argument),
         Some (argument) if argument == "csv" => csv(&argument),
         Some(argument) => Err(Box::from(format!("{} is not correct input argument",argument))),
         None => Err(Box::from(format!("There was no input argument."))),
    };
    
    
    match outtext {
     Err(result) => { 
        if arg.is_some() {
         eprintln!("In method {} occured following error: {}",arg.unwrap(),result)
        } else {
         eprintln!("{}",result)
        }
     },
     Ok(result) => {
      println!("{}",result)
    }
    }
    
}


fn csv(argm:&String) -> Result<String, Box<dyn Error>> {
 if argm != "csv" {
        Err(Box::from( format!("csv function was called, meanwile argument is {}",argm) ))
      } else {   
    
 let mut rdr = csv::Reader::from_reader(io::stdin());
    
   let record = rdr.headers()?;  
   for element in record.iter() {  
    print!("{:16}",element);   
   }
   print!("\n");
 
 for result in rdr.records() {
  let record = result?;
   for element in record.iter() {
     print!("{:16}",element);
   }
   print!("\n")
 }
 Ok(" ".to_string())   
          
    }
}


fn lowercase(argm:&String) -> Result<String, Box<dyn Error>> {
      if argm != "lowercase" {
        Err(Box::from( format!("lowercase function was called, meanwile argument is {}",argm) ))
      } else {
      let mut text = String::new();
                println!("Enter the text you want to convert:");
                io::stdin()
                    .read_line(&mut text)
                    .expect("Failed to read line");
       Ok(text.to_lowercase())
      }
}

fn uppercase(argm:&String) -> Result<String, Box<dyn Error>> {
    if argm != "uppercase" {  
    Err(Box::from( format!("uppercase function was called, meanwile argument is {}",argm) ))
    } else {
      let mut text = String::new();
                println!("Enter the text you want to convert:");
                io::stdin()
                    .read_line(&mut text)
                    .expect("Failed to read line");
    Ok(text.to_uppercase())
    }
}

fn no_space(argm:&String) -> Result<String, Box<dyn Error>> {
    if argm != "no-space" {  
    Err(Box::from( format!("no-space function was called, meanwile argument is {}",argm) ))
    } else {
      let mut text = String::new();
                println!("Enter the text you want to convert:");
                io::stdin()
                    .read_line(&mut text)
                    .expect("Failed to read line");
      Ok(text.replace(" ", ""))
    }
}

fn my_slugify(argm:&String) -> Result<String, Box<dyn Error>> {
    if argm != "slugify" {  
    Err(Box::from( format!("slugify function was called, meanwile argument is {}",argm) ))
    } else {
      let mut text = String::new();
                println!("Enter the text you want to convert:");
                io::stdin()
                    .read_line(&mut text)
                    .expect("Failed to read line");
      Ok(slugify(text))
    }
}

fn camelcase(argm:&String) -> Result<String, Box<dyn Error>> {
    if argm != "camelcase" {  
    Err(Box::from( format!("camelcase function was called, meanwile argument is {}",argm) ))
    } else {
      let mut text = String::new();
                println!("Enter the text you want to convert:");
                io::stdin()
                    .read_line(&mut text)
                    .expect("Failed to read line");
      Ok(AsUpperCamelCase(text).to_string())
    }
}

fn snakecase(argm:&String) -> Result<String, Box<dyn Error>> {
      if argm != "snakecase" {  
    Err(Box::from( format!("snakecase function was called, meanwile argument is {}",argm) ))
    } else {
      let mut text = String::new();
                println!("Enter the text you want to convert:");
                io::stdin()
                    .read_line(&mut text)
                    .expect("Failed to read line");
      Ok(AsSnakeCase(text).to_string())
    }
}


/* fn help() {
    println!("This program takes one argument.");
    println!("");
    println!("Options:");
    println!("lowercase - convert the entire text to lowercase");
    println!("uppercase - convert the entire text to uppercase");
    println!("no-spaces - remove all spaces from the text");
    println!("slugify - convert the text into a slug");
    println!("camelcase - convert the text into a UpperCamelCase");
    println!("snakecase - convert the text into a snake_case");
} */


