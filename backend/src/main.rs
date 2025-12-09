use backend::error::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    println!("Comic Reader Backend starting...");
    Ok(())
}
