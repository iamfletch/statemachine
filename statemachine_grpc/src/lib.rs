pub use hello::*;

mod hello {
  tonic::include_proto!("hello");
}
