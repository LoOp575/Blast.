fn main() {
    cc::Build::new()
        .file("src/recursive.c")
        .compile("recursive_engine");
}
