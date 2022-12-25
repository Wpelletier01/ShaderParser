/*   

    This is a kind of parser for my own engine for getting things like variables,preprocess declarations, etc.
    This is work in progress which updates when I need some information about shader for the engine. This does
    not check for syntax errors

*/



#![allow(dead_code)]
#![allow(non_snake_case)]



use std::path::Path;
use std::fs;
use thiserror::Error;


#[cfg(test)] 
mod test {
    

    
    
    use std::env;
    use super::*;
    

    fn get_relative_path(p:&str) -> String { format!("{}/{}",env::current_dir().unwrap().to_str().unwrap(),p)}

    #[test]
    fn load_correctly() -> Result<(),String> {

        match load_file(get_relative_path("data_test/correct_shader.frag").as_str() ) {
            
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

        let t = load_file("../data_test/..");

        assert!(t.is_err());

    }
    //
    #[test]
    fn shader_type() -> Result<(),String> {

        match load_file(get_relative_path("data_test/correct_shader.frag").as_str()) {


            Ok(sinfo) => {

                if sinfo.type_ ==  ShaderType::FRAGMENT {

                    return Ok(())
                }

                Err(format!("expected shader type FRAGMENT but got {:?}",sinfo.type_))

            },

            Err(err) => Err(format!("unable to compare shader type because of '{}'",err.to_string()))



        }

    }

    #[test]
    fn broken_symlink() {

        let p = get_relative_path("data_test/ex_symlink/test");

        match load_file(p.as_str())  {

            Ok(_) => assert!(false),

            Err(e) => assert_eq!(EParser::LOADING(p.to_string(),"Broken symbolic link".to_string()),e)


        }

    }



    
    // filtering
    
    #[test]
    fn version_preprocess(){


        let test_str = "#version 430 core";

        assert_eq!(
            parse_preprocessor(test_str).unwrap(),
            PreprocessorDeclarationType::VERSION(430_u16,VersionBranch::CORE)
        ); 

    }   

    #[test]
    fn variable_layout() {

        match parse_layout_storage("layout (location = 2) in vec2 aTexCoord;") {

            Some(ctn) => {

                let mut t = LayoutDeclaration::init();

                t.push((LayoutVarType::LOCATION,2));
                

                
                assert_eq!(ctn,StorageQualifier::LAYOUT(t));

            },
            None => assert!(false),

        }


    }





}




#[allow(non_camel_case_types,non_snake_case)]
#[derive(Error,Debug,PartialEq)]
pub enum EParser {
    
    #[error("Unable to loading file {0}. Reason: {1}")]
    LOADING(String,String),
    #[error("Unable to convert OsString to String")]
    OS_STRING_CONVERSION,
    #[error("The extension '{0}' is not suported")]
    UNSUPORTED_EXT(String),
    #[error("The glsl language expect to have #version before anything else but found '{0}'")]
    OMITTED_FIRST_LINE(String),
    #[error("Unable to get index location of &str '{0}'")]
    INDEX_PATERN(String),
    #[error("Unable to parse string to {0}. Reason: {1}")]
    STRING_PARSING(String,String),
    #[error("Cant parse line {0} because of '{1}'")]
    PARSING_LINE(String,String),
    


}
   



fn load_file(fp:&str) -> Result<ShaderFileInfo,EParser> {

    
    let p = Path::new(fp);

    
    match p.try_exists()  {

        Ok(rep) => {
            
            if !rep {

                if p.is_symlink()  {
                    return Err(EParser::LOADING(fp.to_string(),"Broken symbolic link".to_string()));
                }

                return Err(EParser::LOADING(fp.to_string(),"file doesnt exist".to_string()));
                

            }


        },

        Err(e) => return Err(EParser::LOADING(fp.to_string(),e.to_string())),

    }
    
    
    let stype:ShaderType;

    // Check if the file passed have a suported extension
    match p.extension() {
        
        Some(_ext) => {
            
            let ext = match _ext.to_str() {
                Some(ex) => ex,
                None => return Err(EParser::LOADING(fp.to_string(),EParser::OS_STRING_CONVERSION.to_string())),

            };


            match ext {
                
                "frag" => stype = ShaderType::FRAGMENT,
                "vert" => stype = ShaderType::VERTEX,
                _ => return Err(EParser::LOADING(fp.to_string(),EParser::UNSUPORTED_EXT(ext.to_string()).to_string())), 

            }


        },
        
        None => return Err(EParser::LOADING(
                fp.to_string(),
                "unable to retrieve file extension. Possible causes:\n\t- hadn't a file name\n\t- dont have a dot\n\t- have dot but nothing after".to_string()
            ))

    
    }



    let s:Vec<String> = match fs::read_to_string(p) {

        Ok(val) => 
            val.split("\n").filter(|line| line.to_string().trim() != "" ).map(|result| result.to_string()) .collect(),
        Err(e) => return Err(EParser::LOADING(fp.to_string(),e.to_string()))


    };

    

     
    
    let sinfo = ShaderFileInfo::new(
        stype,
        s
    );

    Ok(sinfo)

}


fn filtering(mut shader_info:ShaderFileInfo) -> Result<(),EParser> {

    // check if the first line is a preprocess declaration for the glsl version
    if !shader_info.content[0].contains("#version") {
        return Err(EParser::OMITTED_FIRST_LINE(shader_info.content[0].to_string())); 
    }



    let mut declarations:Vec<DeclarationLine> = Vec::new();



    for line in shader_info.content.iter() {

        let non_whitespace_chars:Vec<char> = line
            .chars()
            .filter(|c| c != &' ')
            .collect();

        if non_whitespace_chars[0] == '#' {

            match parse_preprocessor(line) {

                Ok(decl) => declarations.push(DeclarationLine::PREPROCESSOR(decl)),
     
                Err(e) => return Err(EParser::PARSING_LINE(line.to_string(),e.to_string()))

            }


        } 

        let store_qualifiers = get_storage_qualifier(line);


       

    }
            
    


    Ok(())


}
             

fn get_storage_qualifier(line:&str) -> Vec<StorageQualifier>{

    let mut vstorage:Vec<StorageQualifier> = Vec::new();

    match parse_layout_storage(line) {

        Some(s) => vstorage.push(s),
        None => {}

    }


    
    vstorage

}


fn parse_shade_stage_storage(line:&str) -> Vec<StorageQualifier> {

    let mut vstorage:Vec<StorageQualifier> = Vec::new();

    if line.contains(" in ") {

   

    }
    
    vstorage

}


fn parse_layout_storage(line:&str) -> Option<StorageQualifier> {

    if line.contains("layout") {

        let open_par_index = match line.find("layout") {

            Some(i) => i + 6,
            None => return None

        };

        let mut close_par_index = open_par_index;

        let mut found = false;


        while !found {

            if line.chars().nth(close_par_index).unwrap() == ')' {

                found = true;

            }

            close_par_index += 1;


        }
        
        let par_content:Vec<String> = line.to_string()[open_par_index + 1 .. close_par_index ]
            .split(',')
            .map(|f| f.to_string())
            .collect();
        
        let mut layout_var = LayoutDeclaration::init();   


        for content in par_content.iter() {

            if content.contains("location") {

                let type_ = LayoutVarType::LOCATION;


                let value:String = content
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect();

                let value_num:u32 = match value.parse::<u32>() {

                    Ok(v) => v,
                    Err(_) => return None


                };

                layout_var.push((type_,value_num))


            }



        }
        
        
        return Some(StorageQualifier::LAYOUT(layout_var))


    }


    None 

}

               
fn parse_preprocessor(line:&str) -> Result<PreprocessorDeclarationType,EParser> {

    if line.contains("version") {

        

        let ver:String = line
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        
        let ver_num:u16 = match ver.parse::<u16>() {

            Ok(num) => num,
            Err(e) => return Err(EParser::STRING_PARSING("U16".to_string(),e.to_string()))

        };


        let branch:String = line
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect();

        let index = match branch.find("version") {

            Some(index) => index,
            None => return Err(EParser::INDEX_PATERN(branch.to_string()))

        };

    

        let version_branch = match &branch[index + 7 ..] {
            
            "core"  =>   VersionBranch::CORE,
            _       =>   VersionBranch::UNKNOWN

        };

        return Ok(PreprocessorDeclarationType::VERSION(ver_num,version_branch));

        

        

    }

    unimplemented!()

}

fn get_var_type(line:&str) -> Option<VariableType> {




    None

}



#[derive(Debug,PartialEq)]
pub enum ShaderType {

    VERTEX,
    FRAGMENT,
    TESSCONTROL,
    GEOMETRY

}


pub struct ShaderFileInfo {

    type_: ShaderType,
    content: Vec<String>,     // file content 
    declarations: Vec<DeclarationLine>

}

impl ShaderFileInfo{

    fn new(type_: ShaderType, content: Vec<String> ) -> ShaderFileInfo {
        ShaderFileInfo { type_: type_, declarations: Vec::new(), content: content } 
    }

    fn push_declaration(&mut self, declaration:DeclarationLine) { self.declarations.push(declaration) }

}



#[derive(Debug,PartialEq)]
pub enum DeclarationLine {

    PREPROCESSOR(PreprocessorDeclarationType),
    VARIABLE(ShaderVariables)


}

#[derive(Debug,PartialEq)]
pub enum PreprocessorDeclarationType {


    VERSION(u16,VersionBranch)


}

#[derive(Debug,PartialEq)]
pub enum VersionBranch {

    CORE,
    UNKNOWN
}

#[derive(Debug,PartialEq)]
pub struct ShaderVariables {

    name:       String,
    store_type: Vec<StorageQualifier>,
    var_type:   VariableType,


}





#[derive(Debug,PartialEq)]
pub enum VariableType {


    //scalar types
    BOOL(bool),
    INT(i32),
    UINT(u32),
    FLOAT(f32),
    DOUBLE(f64),


    // vector types
    BVEC2(bool,bool),
    BVEC3(bool,bool,bool),
    BVEC4(bool,bool,bool,bool),
    
    IVEC2(i32,i32),
    IVEC3(i32,i32,i32),
    IVEC4(i32,i32,i32,i32),

    UVEC2(u32,u32),
    UVEC3,
    UVEC4,

    VEC2(f32,f32),
    VEC3(f32,f32,f32),
    VEC4(f32,f32,f32,f32),

    DVEC2(f64,f64),
    DVEC3(f64,f64,f64),
    DVEC4(f64,f64,f64,f64),

    // matrice types
    MAT2(nalgebra_glm::Mat2),
    MAT3(nalgebra_glm::Mat3),
    MAT4(nalgebra_glm::Mat4),





}



#[derive(Debug,PartialEq)]
pub struct LayoutDeclaration { 

    variables: Vec<(LayoutVarType,u32)> 

}

impl LayoutDeclaration {

    fn init() -> Self { LayoutDeclaration { variables: Vec::new() } }

    fn push(&mut self, var:(LayoutVarType,u32)) { self.variables.push(var) }

}


#[derive(Debug,PartialEq)]
pub enum LayoutVarType {

    LOCATION

}




#[derive(Debug,PartialEq)]
pub enum StorageQualifier {

    DEFAULT,
    CONST,
    IN,
    OUT,
    UNIFORM,
    LAYOUT(LayoutDeclaration)



}






//TODO: atomic type 
//TODO: image 

