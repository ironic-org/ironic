use ironic::Injectable;

#[derive(Injectable)]
struct InvalidProvider {
    dependency: String,
}

fn main() {}
