use nawah_core::context::NawahContext;

#[tokio::main]
async fn main() {
    let context = NawahContext::default();
    match context.load_application_via_bollard("C:\\Users\\zacks\\Desktop\\Work", true).await {
        Ok(t) => println!("{}", t),
        Err(e) => println!("{:?}", e)
    };
}
