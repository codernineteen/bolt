async fn lazy_hello() {
    println!("world!");
}

#[tokio::main]
async fn main() {
    let op = lazy_hello();

    println!("hello");
    op.await;
}
