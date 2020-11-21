///
/// Build script for Super Ferris.
/// 

use glob;
use std::fs;

fn main() {
    let mut compiler = shaderc::Compiler::new().expect("Could not create shaderc compiler.");

    glob::glob("src/shaders/*.vert")
        .unwrap()
        .chain(glob::glob("src/shaders/*.frag").unwrap())
        .for_each(|result| {
            let path_buf = result.unwrap();
            let file_path = path_buf.to_str().unwrap();
            let file_contents = std::fs::read_to_string(file_path).expect("Could not open shader source.");
            let kind =
                if path_buf.extension().unwrap().eq("vert") {shaderc::ShaderKind::Vertex}
                else {shaderc::ShaderKind::Fragment};
            println!("cargo:rerun-if-changed={}", path_buf.to_str().unwrap());
                
            match compiler.compile_into_spirv(
                &file_contents[..],
                kind,
                file_path,
                "main",
                None
            ) {
                Ok(x) => {
                    let mut out_file_name = path_buf.to_str().unwrap().to_string();
                    out_file_name.push_str(&String::from(".spirv")[..]);
                    fs::write(out_file_name, x.as_binary_u8()).unwrap();
                },
                Err(x) => {panic!("{}", x.to_string());},
            }
        });
}