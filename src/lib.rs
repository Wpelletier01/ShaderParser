#![allow(dead_code)]
#![allow(non_snake_case)]

use std::path::Path;
use std::fs;
use thiserror::Error;


#[cfg(test)] 
mod test {
    
    use super::*;
    
    #[test]
    fn test_load_correctly() -> Result<(),String> {

        match load_file("/disk/Other/Dev/Rust/Everly/data/shader/parser_test/good_test.frag") {
            
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string())
        }

    }
    
    #[test]
    fn bad_ext() {
        

        let t = load_file("/disk/Other/Dev/Rust/Everly/ShaderParser/data_test/bad_extension.txt");

        assert!(t.is_err());


    }

    #[test]
    fn bad_path() {

        let t = load_file("/disk/Other/Dev/Rust/Everly/ShaderParser/data_test/..");

        assert!(t.is_err());

    }
    


}




#[allow(non_camel_case_types,non_snake_case)]
#[derive(Error,Debug)]
pub enum EParser {
    
    #[error("Unable to loading file {0}. Reason: {1}")]
    LOADING(String,String),
    #[error("Unable to convert OsString to String")]
    OS_STRING_CONVERSION,
    #[error("The extension '{0}' is not suported")]
    UNSUPORTED_EXT(String)
    


}
   



fn load_file(fp:&str) -> Result<String,EParser> {

    
    let p = Path::new(fp);

    
    match p.try_exists()  {

        Ok(rep) => {
            
            if !rep {

                return Err(EParser::LOADING(fp.to_string(),"Broken symbolic link".to_string()));

            }


        },

        Err(e) => return Err(EParser::LOADING(fp.to_string(),e.to_string())),

    }
    
    

    // Check if the file passed have a suported extension
    match p.extension() {
        
        Some(_ext) => {
            
            let ext = match _ext.to_str() {
                Some(ex) => ex,
                None => return Err(EParser::LOADING(fp.to_string(),EParser::OS_STRING_CONVERSION.to_string())),

            };


            match ext {
                
                "frag" => {},
                "vert" => {},
                _ => return Err(EParser::LOADING(fp.to_string(),EParser::UNSUPORTED_EXT(ext.to_string()).to_string())), 

            }


        },
        
        None => return Err(EParser::LOADING(
                fp.to_string(),
                "unable to retrieve file extension. Possible causes:\n\t- hadn't a file name\n\t- dont have a dot\n\t- have dot but nothing after".to_string()
                ))

    
    }

    let s = fs::read_to_string(p).map_err(|e| EParser::LOADING(fp.to_string(),e.to_string())).unwrap();
    
    Ok(s)

}



             
            

               


















