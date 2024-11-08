use anyhow::Result;
use std::fs;
fn main() -> Result<()> {
    // 如果目录不存在则创建
    fs::create_dir_all("src/pb")?;
    let builder = tonic_build::configure();
    builder
        .out_dir("src/pb")
        .compile_protos(&["../protos/crm.proto"], &["../protos"])?;
    Ok(())
}
// fn main() -> Result<()> {
//     // 如果目录不存在则创建
//     fs::create_dir_all("src/pb")?;
//     let mut config = prost_build::Config::new();
//     config
//         .out_dir("src/pb")
//         .compile_protos(&["../protos/crm.proto"], &["../protos"])?;
//     Ok(())
// }
