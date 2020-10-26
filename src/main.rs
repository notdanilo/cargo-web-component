use std::process;
use std::fs::File;
use std::fs::create_dir_all;
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

use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};
use std::path::PathBuf;

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let mut path: PathBuf = req.match_info().query("filename").parse().unwrap();
    if path.display().to_string() == "" {
        path = "index.html".parse().unwrap();
    }
    Ok(NamedFile::open(path)?)
}

use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let out_dir = "pkg";
    create_dir_all(format!("./{}/web-component", out_dir)).expect("Couldn't create all directories.");

    match Command::from_args() {
        Command::Build => {
            process::Command::new("wasm-pack")
                .arg("build")
                .arg("--target")
                .arg("web")
                .arg("--no-typescript")
                .arg("--out-dir")
                .arg(out_dir)
                .status()
                .expect("Failed");

            let contents = std::fs::read_to_string("./Cargo.toml")
                .expect("Something went wrong reading the file");
            let toml = contents.parse::<Value>().unwrap();

            let package_name = toml["package"]["name"].as_str().expect("Couldn't get package name.");

            //assert_eq!(value["foo"].as_str(), Some("bar"));

            let index_content = include_str!("../dependencies/index.html");
            let index_content = index_content.replace("PACKAGE-NAME", package_name);

            write_to(index_content.as_bytes()                                                          , &format!("./{}/index.html", out_dir));
            write_to(include_bytes!("../dependencies/web-component/vue.js")                            , &format!("./{}/web-component/vue.js", out_dir));
            write_to(include_bytes!("../dependencies/web-component/web-component.js")                  , &format!("./{}/web-component/web-component.js", out_dir));
            write_to(include_bytes!("../dependencies/web-component/web-component-definition.js")       , &format!("./{}/web-component/web-component-definition.js", out_dir));
            write_to(include_bytes!("../dependencies/web-component/web-component-javascript.js")       , &format!("./{}/web-component/web-component-javascript.js", out_dir));
            write_to(include_bytes!("../dependencies/web-component/web-component-javascript-loader.js"), &format!("./{}/web-component/web-component-javascript-loader.js", out_dir));
            write_to(include_bytes!("../dependencies/web-component/web-component-loader.js")           , &format!("./{}/web-component/web-component-loader.js", out_dir));
            write_to(include_bytes!("../dependencies/web-component/web-component-wasm.js")             , &format!("./{}/web-component/web-component-wasm.js", out_dir));
            write_to(include_bytes!("../dependencies/web-component/web-component-wasm-loader.js")      , &format!("./{}/web-component/web-component-wasm-loader.js", out_dir));
        },
        Command::Serve(port) => {
            HttpServer::new(|| App::new().route("/{filename:.*}", web::get().to(index)))
                .bind(format!("127.0.0.1:{}", port))?
                .run()
                .await?;
        },
        _ => ()
    }

    Ok(())
}
