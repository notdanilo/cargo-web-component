use std::process;
use std::fs::File;
use std::io::Write;
use std::env;
use toml::Value;

fn write_to(bytes:&[u8], path:&str) {
    let mut index_file = File::create(path).expect("Couldn't create file");
    index_file.write_all(bytes).expect("File writing failed.");
}

enum Command {
    Serve(u16),
    Build,
    None
}

impl Command {
    fn from_args() -> Self {
        let args: Vec<String> = env::args().collect();
        if let Some(last) = args.last() {
            if last == "serve" {
                Command::Serve(8080)
            } else if last == "build" {
                Command::Build
            } else {
                Command::None
            }
        } else {
            Command::None
        }
    }
}

use actix_files as fs;
use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let out_dir = "pkg";

    match Command::from_args() {
        Command::Build => {
            let contents = std::fs::read_to_string("./Cargo.toml")
                .expect("Something went wrong reading the file");
            let toml = contents.parse::<Value>().unwrap();

            let package_name = toml["package"]["name"].as_str().expect("Couldn't get package name.");

            process::Command::new("wasm-pack")
                .arg("build")
                .arg("--target")
                .arg("web")
                .arg("--no-typescript")
                .arg("--out-dir")
                .arg(out_dir)
                .arg("--out-name")
                .arg(package_name)
                .status()
                .expect("Failed");

            let index_content = include_str!("../dependencies/index.html");
            let index_content = index_content
                .replace("PACKAGE-NAME", package_name)
                .replace("WEB-COMPONENT-SOURCE", "https://notdanilo.github.io/web-component/web-component.js");

            write_to(index_content.as_bytes(), &format!("./{}/index.html", out_dir));
        },
        Command::Serve(port) => {
            println!("Serving at localhost:{}", port);
            HttpServer::new(move || {
                App::new().service(
                    fs::Files::new("/", out_dir)
                        .show_files_listing()
                        .use_last_modified(true)
                        .index_file("index.html"),
                )
            })
                .bind(format!("127.0.0.1:{}", port))?
                .run()
                .await?;
        },
        _ => {
            println!("cargo web-component <command>");
            println!("Where command is:");
            println!("build - to build the project.");
            println!("serve - to build and serve the project.");
        }
    }

    Ok(())
}
