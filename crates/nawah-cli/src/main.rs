use nawah_core::context::{AppNotFound, NawahContext};

fn main() {
    let context = NawahContext::default();
    context.load_application("/Users/mishal/Projects/elm-wallet", true);
}
