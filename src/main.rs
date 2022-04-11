#[tokio::main]
async fn main() {
  match mockery::get_args() {
    Err(e) => eprintln!("{e}"),
    Ok(config) => {
      if let Err(e) = mockery::run(config).await {
        eprintln!("{e}");
      }
    }
  }
}
