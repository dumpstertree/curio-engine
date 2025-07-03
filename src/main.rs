use pollster::FutureExt;
use tutorial3_pipeline::run;

fn main() {
    if run().is_err() {
        panic!("has error");
    }
}
