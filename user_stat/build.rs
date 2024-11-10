use anyhow::Result;
use proto_builder_trait::tonic::BuilderAttributes;
use std::fs;

fn main() -> Result<()> {
    // 如果目录不存在则创建
    fs::create_dir_all("src/pb")?;
    // 实例化 BuilderAttributes
    let builder = tonic_build::configure();
    // 设置输出目录
    builder
        .out_dir("src/pb")
        // 生成时加入Serde的宏
        .with_serde(
            &["User"],
            true,
            true,
            Some(&[r#"#[serde(rename_all = "camelCase")]"#]),
        )
        // 生成时加入 Sqxl 宏
        .with_sqlx_from_row(&["User"], None)
        // 假如 derive_builder 宏可用，则生成 builder 配置
        .with_derive_builder(
            &[
                "User",
                "QueryRequest",
                "RawQueryRequest",
                "TimeQuery",
                "IdQuery",
            ],
            None,
        )
        // 处理字段属性
        .with_field_attributes(
            &["User.email", "User.name", "RawQueryRequest.query"],
            &[r#"#[builder(setter(into))]"#],
        )
        // 处理时间字段属性
        .with_field_attributes(
            &["TimeQuery.before", "TimeQuery.after"],
            &[r#"#[builder(setter(into,strip_option))]"#],
        )
        .with_field_attributes(
            &["QueryRequest.timestamps"],
            &[r#"#[builder(setter(each(name="timestamp",into)))]"#],
        )
        .with_field_attributes(
            &["QueryRequest.ids"],
            &[r#"#[builder(setter(each(name="id",into)))]"#],
        )
        .compile_protos(
            &[
                "../protos/user_stats/messages.proto",
                "../protos/user_stats/rpc.proto",
            ],
            &["../protos"],
        )?;
    Ok(())
}

// fn main() -> Result<()> {
//     // 如果目录不存在则创建
//     fs::create_dir_all("src/pb")?;
//     let builder = tonic_build::configure();
//     builder.out_dir("src/pb").compile_protos(
//         &[
//             "../protos/user_stats/messages.proto",
//             "../protos/user_stats/rpc.proto",
//         ],
//         &["../protos"],
//     )?;
//     Ok(())
// }
