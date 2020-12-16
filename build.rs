use std::fs::write;

fn main() {
    println!("cargo:rerun-if-changed=src/shader.vert");
    println!("cargo:rerun-if-changed=src/shader.frag");
    let vertex_shader_source = include_str!("src/shader.vert");
    let fragment_shader_source = include_str!("src/shader.frag");
    let mut shader_compiler = shaderc::Compiler::new().unwrap(); 
    let vertex_shader_spirv = shader_compiler.compile_into_spirv(vertex_shader_source, shaderc::ShaderKind::Vertex, "src/shader.vert", "main", None).unwrap();
    let fragment_shader_spirv = shader_compiler.compile_into_spirv(fragment_shader_source, shaderc::ShaderKind::Fragment, "src/shader.frag", "main", None).unwrap();
    write("src/shader.vert.spv", vertex_shader_spirv.as_binary_u8()).unwrap();
    write("src/shader.frag.spv", fragment_shader_spirv.as_binary_u8()).unwrap();
}
