use crate::dependency::Dependencies;

pub fn address() {
    let deps = Dependencies::default();
    println!("{:?}", deps.address);
}