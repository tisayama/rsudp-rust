fn main() {
    prost_build::compile_protos(
        &["proto/seismic.proto"],
        &["proto/"],
    ).unwrap();
}
