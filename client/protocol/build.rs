use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &["/Users/caelum/CLionProjects/riposte/proto/riposte.proto"],
        &["/Users/caelum/CLionProjects/riposte/proto/"],
    )?;
    Ok(())
}
