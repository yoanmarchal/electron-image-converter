// Icône et métadonnées de l'exécutable Windows (Explorateur, barre des tâches)
fn main() {
    println!("cargo::rerun-if-changed=assets/icons/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/icons/icon.ico")
            .set("ProductName", "Image Converter")
            .set("FileDescription", "Image Converter")
            .compile()
            .expect("ressources Windows");
    }
}
