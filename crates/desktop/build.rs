fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("../../assets/colorcraft.ico");
        resource.set("ProductName", "ColorCraft");
        resource.set("FileDescription", "ColorCraft — laboratório de paletas");
        resource.set("LegalCopyright", "ColorCraft contributors");
        if let Err(error) = resource.compile() {
            eprintln!("Falha ao compilar o recurso do aplicativo: {error}");
            std::process::exit(1);
        }
    }
}
