
//
/// The different type of shader file program
#[derive(Debug,PartialEq)]
pub enum ShaderType {

    VERTEX,
    FRAGMENT,
    TESSCONTROL,
    GEOMETRY

}
//
//
/// These are the different types of declaration that this library will store 
/// in the shader info struct 
#[derive(Debug,PartialEq)]
pub enum DeclarationLine {

    PREPROCESSOR(PreprocessorDeclarationType),
    VARIABLE(ShaderVariables)

}
//
//
/// The variable data types that GLSL accepts. They store also the variable value if the 
/// program declares data to it. Otherwise, they store nothing.
#[derive(Debug,PartialEq)]
pub enum VariableType {

    //scalar types
    BOOL    (Option<bool>),
    INT     (Option<i32>),
    UINT    (Option<u32>),
    FLOAT   (Option<f32>),
    DOUBLE  (Option<f64>),

    // vector types
    BVEC2(Option<[bool;2]>),
    BVEC3(Option<[bool;3]>),
    BVEC4(Option<[bool;4]>),
    
    IVEC2(Option<[i32;2]>),
    IVEC3(Option<[i32;3]>),
    IVEC4(Option<[i32;4]>),

    UVEC2(Option<[u32;2]>),
    UVEC3(Option<[u32;3]>),
    UVEC4(Option<[u32;4]>),

    VEC2(Option<[f32;2]>),
    VEC3(Option<[f32;3]>),
    VEC4(Option<[f32;4]>),

    DVEC2(Option<[f64;2]>),
    DVEC3(Option<[f64;3]>),
    DVEC4(Option<[f64;4]>),

    // matrices types
    MAT2(Option<[[f32;4]; 2]>),
    MAT3(Option<[[f32;4]; 3]>),
    MAT4(Option<[[f32;4]; 4]>),

    // Double precision matrices types
    DMAT2(Option<[[f64;4]; 2]>),
    DMAT3(Option<[[f64;4]; 3]>),
    DMAT4(Option<[[f64;4]; 4]>),

    // texture sampler types
    SAMPLER2D(Option<f32>),
    USAMPLER2D(Option<u32>),
    ISAMPLER2D(Option<i32>),

    //TODO: add other type when it will be needed like atomic and Image 

}
//
//
// ------------------------------------------------------------------------------------------
//  Preprocessor Declarations types struct 
//
//
/// The different possible preprocessor declarations
#[derive(Debug,PartialEq)]
pub enum PreprocessorDeclarationType {

    VERSION(u16,VersionBranch)

}
//
//
/// The different type of specifier that are suppose to come with the glsl compiler version
#[derive(Debug,PartialEq)]
pub enum VersionBranch {

    CORE,
    UNKNOWN

}
//
//
// ------------------------------------------------------------------------------------------
// Storage qualifiers declaration
//
//
/// the different types of storage qualifiers possible
#[derive(Debug,PartialEq)]
pub enum StorageQualifier {

    DEFAULT,
    CONST,
    IN,
    OUT,
    UNIFORM,
    LAYOUT(LayoutDeclaration)


}
//
impl StorageQualifier {
    //
    /// return how its declared in the shader source file. It make easier to filter the line
    /// contents 
    pub(crate) fn as_str(&self) -> &str {

        match self {

            Self::CONST =>      "const",
            Self::DEFAULT =>    "",
            Self::IN =>         "in",
            Self::OUT =>        "out",
            Self::LAYOUT(layout) => layout.raw.as_str(),
            Self::UNIFORM =>    "uniform",
        
        }

    }
    //
}
//
//
/// Store the declaration in parentheses when the line content have a layout storage 
/// declaration. 
///     example of what this struct should store in: 
///         'layout (location = 2) in vec2 aTexCoord;'
///             - raw => 'layout (location = 2)',
///             - variables => '(location = 2)' 
///        
#[derive(Debug,PartialEq)]
pub struct LayoutDeclaration { 

    raw:       String,
    variables: Vec<(LayoutVarType,u32)> 

}
//
impl LayoutDeclaration {
    //
    /// Initialize the struct with an empty vector ready to have some stuff in it
    pub(crate) fn init(raw:&str) -> 
        Self { LayoutDeclaration { variables: Vec::new(),raw: raw.to_string() } }
    
        
    /// Add (LayoutVarType,u32) at the end of the vector variables  
    pub(crate) fn push(&mut self, var:(LayoutVarType,u32)) { self.variables.push(var) }
    //
}
//
//
/// Type of variable possible that could be declared in the parentheses of a layout 
/// declaration
#[derive(Debug,PartialEq)]
pub enum LayoutVarType {

    LOCATION,
    BINDING,
    COMPONENT,

    //TODO: add other possible declarations see:
    //   https://www.khronos.org/opengl/wiki/Layout_Qualifier_(GLSL)

}
//
//
// ------------------------------------------------------------------------------------------
//
//
/// Store the variable declared in the a shader file 
/// This is what the shader info struct will store 
#[derive(Debug,PartialEq)]
pub struct ShaderVariables {

    name:       String,
    store_type: Vec<StorageQualifier>,
    var_type:   VariableType,


}



