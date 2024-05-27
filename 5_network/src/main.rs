mod csv_mod;
use heck::AsSnakeCase;
use heck::AsUpperCamelCase;
use slug::slugify;
use std::env;
use std::io;
use std::error::Error;
use crate::csv_mod::csv_mod::read_csv;
use std::thread;


/*
Set up Concurrency:

    Spin up two threads: one dedicated to receiving input and another for processing it.
    Make use of channels to transfer data between the two threads. You can employ Rust's native std::mpsc::channel or explore the flume library for this.
  
  
Input-Receiving Thread:

    This thread should continuously read from stdin and parse the received input in the format <command> <input>. Remember to avoid "stringly-typed APIs" - the command can be an enum. For an enum, you can implement the FromStr trait for idiomatic parsing.


  
*/

// Stačí jedno vlákno? Založím vlákno na přijímání vstupů a hlavní vlákno bude to druhé?
// Zatím zakomentuji kód a poté jej odkomentuji, až budu mít hotové daná dvě vlákna
// Pokud jsou argumenty, jedeme klasicky, pokud nejsou vstupní argumenty, spouští se druhé vláno pro přijímání standartního vstupu
// Udělá se to tak, že ve spawnutém vláknu na kontrolu programu se budou načítat i argumenty, pokud jsou vstupní argumenty, požádá se o zadání vstupu, zavolá se 
fn main() {
  
  let handle = thread::spawn(|| {
   println!("Ahoj z vlákna handle!");
  }); 
    handle.join().unwrap();
    let _arg = env::args().nth(1);
    

    /*
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
   
// let rdr=mod_csv::Csv{vec!{"bla1".to_string,"bla2".to_string,"bla3".to_string},vec!{"1".to_string,"2".to_string,"3".to_string}};       
   let filename="csv_example.txt".to_string();     
   let rdr = read_csv(filename);
   println!("{}",rdr.to_string());
/* let mut rdr = csv::Reader::from_reader(io::stdin());
    
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
 */
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
    
    */
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


