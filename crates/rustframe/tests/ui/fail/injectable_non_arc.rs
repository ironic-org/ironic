use rustframe::Injectable;

#[derive(Injectable)]
struct InvalidProvider {
    dependency: String,
}

fn main() {}
