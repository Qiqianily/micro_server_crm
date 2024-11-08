use anyhow::Result;
use crm::User;
use prost::Message;

fn main() -> Result<()> {
    // 创建一个 User 对象
    let user: User = User::new(1, "Alice", "alice@example.com");
    // 编码为 Protobuf 格式
    let encoded = user.encode_to_vec();
    // 打印结果
    println!("user: {:?}, encoded: {:?}", user, encoded);
    // 解码为 User 对象
    let decoded = User::decode(encoded.as_slice())?;
    // 打印结果
    println!("decoded: {:?}", decoded);
    Ok(())
}
