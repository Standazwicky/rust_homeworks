pub mod csv_mod {
    
    use std::fmt;
    use std::fs;
    use std::cmp;
    //use std::io::prelude::*;
    
    pub struct Csv {
      pub header : Vec<String>,
      pub data   : Vec<String>,
    }
    
    impl fmt::Display for Csv {
     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
         let longesth = self.header.iter().map(|x| x.len()).max().unwrap();
         let longestd = self.data.iter().map(|x| x.len()).max().unwrap();
         let longest=cmp::max(longestd,longesth);
         let fixed_length=longest+2;
         println!("{},{},{}",longesth,longestd,longest);
         
         let mut buffer=String::new();
         
         // Format header
         for word in &self.header {
           let formatted_word = format!("{:<width$}", word, width = fixed_length);
           buffer.push_str(&formatted_word);
         }
         
         // Format data
         buffer.push_str("\n");
         println!("délka header {}",self.header.len());
         for (i,word) in self.data.iter().enumerate() {
          let formatted_word = format!("{:<width$}", word, width = fixed_length);
           buffer.push_str(&formatted_word);
           if (i+1) % self.header.len() == 0 {
             buffer.push('\n');
           }
         }
            
         write!(f,"{}", buffer)
     }
    }
    
    pub fn read_csv(filename:String) -> Csv {
        let csvcontent = fs::read_to_string(filename);
        let mut head: Vec<String> = vec![];
        let mut dat: Vec<String> = vec![];
        
        //csvcontent.unwrap().lines()
        let mut isfirst=true; //reading header
        for line in csvcontent.unwrap().lines() {
          if isfirst {
            isfirst = false;
             for word in line.split(",") {  
              head.push(word.to_string());
             }
          } else {
          for word in line.split(",") {  
            dat.push(word.to_string());
          }
        }
        }
  //      println!("{}",csvcontent.unwrap());
        Csv{header:head,data:dat}
    }

}

