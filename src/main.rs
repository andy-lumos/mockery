fn main() {
  if let Err(e) = mockery::get_args().and_then(mockery::run) {
    eprintln!("{e}");
  }
}
