use tagg::app;

fn main() {
    if let Err(err) = app::run() {
        println!("{}", err);
    }
}
