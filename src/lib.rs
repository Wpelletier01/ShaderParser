#![allow(dead_code)]
#![allow(non_snake_case)]


use std::path::Path;
use std::fs;
use std::ptr::eq;
use thiserror::Error;

pub mod datatype;


use datatype::*;


//
//
// ------------------------------------------------------------------------------------------
// Test Section
//
#[cfg(test)] 
mod test {
    
    use std::env;
    use super::*;

    

    fn get_relative_path(p:&str) -> 
        String { format!("{}/{}",env::current_dir().unwrap().to_str().unwrap(),p)}

    #[test]
    fn load_correctly() -> Result<(),String> {

        match load_file(get_relative_path("data_test/correct_shader.frag").as_str() ) {
            
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string())
        }

    }
    
    #[test]
    fn bad_ext() {
        

        let t = 
            load_file(
                "/disk/Other/Dev/Rust/Everly/ShaderParser/data_test/bad_extension.txt"
            );

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

            Err(err) => Err(
                format!("unable to compare shader type because of '{}'",err.to_string())
            )


        }

    }


    #[test]
    fn broken_symlink() {

        let p = get_relative_path("data_test/ex_symlink/test");

        match load_file(p.as_str())  {

            Ok(_) => assert!(false),

            Err(e) => assert_eq!(
                EParser::LOADING(p.to_string(),"Broken symbolic link".to_string()),e
            )


        }

    }



    
    // filtering
    
    #[test]
    fn version_preprocessor(){


        let test_str = "#version 430 core";

        assert_eq!(
            parse_preprocessor(test_str).unwrap(),
            PreprocessorDeclarationType::VERSION(430_u16,VersionBranch::CORE)
        ); 

    }   

    #[test]
    fn get_storage_qualifiers() {

        let mut t = LayoutDeclaration::init("layout (location = 2) in vec2 aTexCoord;");

        t.push((LayoutVarType::LOCATION,2));



        let expected:Vec<StorageQualifier> = vec![

            StorageQualifier::LAYOUT(t),
            StorageQualifier::IN,

        ];

        let founded = 
            get_storage_qualifier("layout (location = 2) in vec2 aTexCoord;");


        assert_eq!(expected,founded);


    }


    #[test]
    fn filtering_storage_qualifier() {
        
        let mut t = LayoutDeclaration::init("layout (location = 2)");

        t.push((LayoutVarType::LOCATION,2));


        assert_eq!(

            remove_storage_type(
                "layout (location = 2) in vec2 aTexCoord;", 
                vec![StorageQualifier::LAYOUT(t),StorageQualifier::IN,]).to_string().trim(),
            String::from("vec2 aTexCoord;")
        
        );


    }




}
//
// Error handling for the library
//
#[allow(non_camel_case_types,non_snake_case)]
#[derive(Error,Debug,PartialEq)]
pub enum EParser {
    
    #[error("Unable to loading file {0}. Reason: {1}")]
    LOADING(String,String),
    #[error("Unable to convert OsString to String")]
    OS_STRING_CONVERSION,
    #[error("The extension '{0}' is not supported")]
    UNSUPPORTED_EXT(String),
    #[error("The glsl language expect to have #version before anything else but found '{0}'")]
    OMITTED_FIRST_LINE(String),
    #[error("Unable to get index location of &str '{0}'")]
    INDEX_PATERN(String),
    #[error("Unable to parse string to {0}. Reason: {1}")]
    STRING_PARSING(String,String),
    #[error("Cant parse line {0} because of '{1}'")]
    PARSING_LINE(String,String),
    


}
   

const TYPE_IN_STR: [&str;27] = [
    
    "bool",
    "int",
    "uint",
    "float",
    "double",


    "uvec2",
    "uvec3",
    "uvec4",

    "ivec2",
    "ivec3",
    "ivec4",

    "bvec2",
    "bvec3",
    "bvec4",

    "vec2",
    "vec3",
    "vec4",

    "dvec2",
    "dvec3",
    "dvec4",

    "mat2",
    "mat3",
    "mat4",
  
    
    "dmat2",
    "dmat3",
    "dmat4",

    "sampler2D"

];


fn load_file(fp:&str) -> Result<ShaderFileInfo,EParser> {

    
    let p = Path::new(fp);

    
    match p.try_exists()  {

        Ok(rep) => {
            
            if !rep {

                if p.is_symlink()  {
                    return Err(EParser::LOADING(fp.to_string(),"Broken symbolic link".to_string()));
                }

                return Err(EParser::LOADING(fp.to_string(),"file doesn't exist".to_string()));
                

            }


        },

        Err(e) => return Err(EParser::LOADING(fp.to_string(),e.to_string())),

    }
    
    
    let stype:ShaderType;

    // Check if the file passed have a supported extension
    match p.extension() {
        
        Some(_ext) => {
            
            let ext = match _ext.to_str() {
                Some(ex) => ex,
                None => return Err(EParser::LOADING(fp.to_string(),EParser::OS_STRING_CONVERSION.to_string())),

            };


            match ext {
                
                "frag" => stype = ShaderType::FRAGMENT,
                "vert" => stype = ShaderType::VERTEX,
                _ => return Err(EParser::LOADING(fp.to_string(),EParser::UNSUPPORTED_EXT(ext.to_string()).to_string())), 

            }


        },
        
        None => return Err(EParser::LOADING(
                fp.to_string(),
                "unable to retrieve file extension. Possible causes:\n\t- hadn't a file name\n\t- don't have a dot\n\t- have dot but nothing after".to_string()
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
        
        let filter_line = remove_storage_type(line, store_qualifiers);



       

    }
            
    


    Ok(())


}
             

fn get_storage_qualifier(line:&str) -> Vec<StorageQualifier>{

    let mut vstorage:Vec<StorageQualifier> = Vec::new();

    if line.contains("layout") {
        
        match parse_layout_storage(line) {

            Some(s) => vstorage.push(s),
            None => {}
    
        }
    }

    if line.contains(" uniform ") {

        vstorage.push(StorageQualifier::UNIFORM);
    
    } else {

        if line.contains(" in ") {
        
            vstorage.push(StorageQualifier::IN);
    
        } else if line.contains(" out ") {
    
            vstorage.push(StorageQualifier::OUT)
    
        } 

    }

    vstorage

}






fn parse_shader_stage_storage(line:&str) -> Option<StorageQualifier> {


    if line.contains(" in ") { return Some(StorageQualifier::IN); }
    else if line.contains(" out") { return Some(StorageQualifier::OUT); }

    None

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
        
        let mut layout_var = LayoutDeclaration::init(line);   


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

    
    let split_line = line.to_string();



    for l in split_line.split(" ") {

        if TYPE_IN_STR.contains(&l) {

            let vtype:VariableType = match l {

                "bool"  => {
                    if have_declared_value(line) {

                        let equal_split = match line.split("=").last() {

                            Some(v) => v,
                            None => return None


                        };

                        if equal_split.contains("false") {
                            
                            VariableType::BOOL(Some(false));

                        } else if equal_split.contains("true") {

                            VariableType::BOOL(Some(true));

                        }


                    }

                    VariableType::BOOL(None)


                },
                "int"   => {
                    if have_declared_value(line) {

                        let equal_split:String = match line.split("=").last() {

                            Some(v) => v.to_string(),
                            None => return None


                        };

                        match equal_split.parse::<i32>() {

                            Ok(value) => VariableType::INT(Some(value)),
                            Err(_) => VariableType::INT(None)

                        };

                        


                    }

                    VariableType::INT(None)

                },
                "uint"  => {

                    if have_declared_value(line) {

                        let equal_split:String = match line.split("=").last() {

                            Some(v) => v.to_string(),
                            None => return None


                        };

                        match equal_split.parse::<u32>() {

                            Ok(value) => VariableType::UINT(Some(value)),
                            Err(_) => VariableType::UINT(None)

                        };

                    }

                    VariableType::UINT(None)

                },
                "float" => {
                    if have_declared_value(line) {

                        let equal_split:String = match line.split("=").last() {

                            Some(v) => v.to_string(),
                            None => return None


                        };

                        match equal_split.parse::<f32>() {

                            Ok(value) => VariableType::FLOAT(Some(value)),
                            Err(_) => VariableType::FLOAT(None)

                        };

                    }

                    VariableType::FLOAT(None)


                },
                "double"=> {
                    if have_declared_value(line) {

                        let equal_split:String = match line.split("=").last() {

                            Some(v) => v.to_string(),
                            None => return None


                        };

                        match equal_split.parse::<f64>() {

                            Ok(value) => VariableType::DOUBLE(Some(value)),
                            Err(_) => VariableType::DOUBLE(None)

                        };

                    }

                    VariableType::DOUBLE(None)

                },
            
            
                "uvec2" => {

                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::UVEC2(None)

                },
                "uvec3" => {
                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::UVEC3(None)

                },
                "uvec4" => {

                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::UVEC4(None)

                },
            
                "ivec2" => {

                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::IVEC2(None)

                },
                "ivec3" => {
                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::IVEC3(None)

                },
                "ivec4" => {

                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::IVEC4(None)

                },

                "bvec2" => {
                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::BVEC2(None)

                },
                "bvec3" => {
                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::BVEC3(None)

                },
                "bvec4" => {
                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::BVEC4(None)

                },
                "vec2"  => {

                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::VEC2(None)

                },
                "vec3"  => {

                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::VEC3(None)

                },
                "vec4"  => {

                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::VEC4(None)


                },
                "dvec2" => {
                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::DVEC2(None)

                },
                "dvec3" => {

                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::DVEC3(None)

                },
                "dvec4" => {

                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::DVEC4(None)

                },
                "mat2"  => {

                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::MAT2(None)


                },
                "mat3"  => {
                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::MAT3(None)

                },
                "mat4"  => {

                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::MAT4(None)


                },
             
                
                "dmat2" => {

                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::DMAT2(None)

                },
                "dmat3" => {

                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::DMAT3(None)


                },
                "dmat4" => {

                    if have_declared_value(line) {

                        unimplemented!()

                    }

                    VariableType::DMAT4(None)


                },

                "sampler2D" => {

                    if have_declared_value(line) {

                        unimplemented!()    
                    }

                    VariableType::SAMPLER2D(None)

                },

                _ => return None,

            };

            return Some(vtype);

        }

        return None 

    }


    None

}


fn have_declared_value(line:&str) -> bool { line.contains("=") }


fn remove_storage_type<'a>(line:&str,vstorage:Vec<StorageQualifier>) -> String {

    let mut filter_line = line.to_string();

    for store in vstorage.iter() {
 
        filter_line = filter_line.replace(&store.as_str(), "");

    }

    filter_line

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













