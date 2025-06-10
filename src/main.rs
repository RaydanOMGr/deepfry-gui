use std::io::Cursor;
use std::{mem};
use std::path::Path;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use clap_builder::ValueEnum;
use deepfry::{deepfry, ChangeMode, Preset};
use deepfry::DeepfryAlgorithm::BitChange;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::document::eval;
use dioxus::prelude::*;
use image::RgbImage;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    LaunchBuilder::new().with_cfg(create_cfg()).launch(App);
}

fn create_cfg() -> Config {
    Config::default().with_window(WindowBuilder::new()
        .with_title("deepfry-gui")
    )
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Main {}
    }
}

#[component]
pub fn Main() -> Element {
    let modes = deepfry::ChangeMode::value_variants();
    let mut selected_mode = use_signal(|| deepfry::ChangeMode::value_variants()[0].clone());
    let mut file_path = use_signal(|| "".to_string());
    let mut file_name = use_signal(|| "".to_string());

    let mut red_value = use_signal(|| 0u8);
    let mut green_value = use_signal(|| 0u8);
    let mut blue_value = use_signal(|| 0u8);

    let mut output = use_signal(|| "".to_string());

    let modes_rendered = modes.iter()
        .map(|mode| {
            if cmp_enum(mode, &selected_mode()) {
                rsx! {
                    option {
                        value: "{mode}",
                        selected: true,
                        "{mode}"
                    }
                }
            } else {
                rsx! {
                    option {
                        value: "{mode}",
                        "{mode}"
                    }
                }
            }
        });

    rsx!(
        div {
            h1 {
                "Deepfry GUI"
            }
        }
        div {
            class: "color-row",

            label {
                r#for: "select_mode",
                "Mode"
            }
            select {
                id: "select_mode",
                onchange: move |evt| {
                    match ChangeMode::from_string(&*evt.value()) {
                        Ok(mode) => selected_mode.set(mode),
                        Err(e) => {
                            eprintln!("Failed to parse mode from string '{}': {:?}", evt.value(), e);
                        }
                    }
                },
                {modes_rendered}
            }
        }
        div {
            class: "color-row",

            label { r#for: "red", "Red" }

            input {
                r#type: "range",
                id: "red",
                min: "0",
                max: "255",
                value: "{red_value}",
                oninput: move |evt| {
                    if let Ok(val) = evt.value().parse::<u8>() {
                        red_value.set(val);
                    }
                },
            }

            input {
                class: "color-text-input",
                maxlength: "3",
                value: "{red_value}",
                onchange: move |evt| {
                    if let Ok(val) = evt.value().parse::<i32>() {
                        red_value.set(val.clamp(0, 255) as u8);
                    }
                }
            }
        }
        div {
            class: "color-row",

            label { r#for: "green", "Green" }

            input {
                r#type: "range",
                id: "green",
                min: "0",
                max: "255",
                value: "{green_value}",
                oninput: move |evt| {
                    if let Ok(val) = evt.value().parse::<u8>() {
                        green_value.set(val);
                    }
                },
            }

            input {
                class: "color-text-input",
                maxlength: "3",
                value: "{green_value}",
                onchange: move |evt| {
                    if let Ok(val) = evt.value().parse::<i32>() {
                        green_value.set(val.clamp(0, 255) as u8);
                    }
                }
            }
        }
        div {
            class: "color-row",

            label { r#for: "blue", "Blue" }

            input {
                r#type: "range",
                id: "blue",
                min: "0",
                max: "255",
                value: "{blue_value}",
                oninput: move |evt| {
                    if let Ok(val) = evt.value().parse::<u8>() {
                        blue_value.set(val);
                    }
                },
            }

            input {
                class: "color-text-input",
                maxlength: "3",
                value: "{blue_value}",
                onchange: move |evt| {
                    if let Ok(val) = evt.value().parse::<i32>() {
                        blue_value.set(val.clamp(0, 255) as u8);
                    }
                }
            }
        }
        div {
            class: "color-row",

            label {
                r#for: "select_file",
                "File"
            }
            input {
                id: "select_file",
                r#type: "file",
                accept: "image/*", // changed from image/* temporarily
                style: "display: none;",
                onchange: move |evt| {
                    if let Some(files) = evt.files() {
                        if let Some(file) = files.files().get(0) {
                            let path = file.as_str();
                            file_path.set(path.to_string());
                            println!("Set path {}", path);
                            let path = Path::new(path);
                            if let Some(name) = path.file_name() {
                                file_name.set(name.to_str().unwrap().to_string());
                            }
                        }
                    }
                }
            }
            button {
                id: "select_file_true",
                onclick: move |evt| {
                    eval("document.getElementById(\"select_file\").click();");
                },
                "Select file"
            }
        }
        div {
            class: "color-row",
            label {
                r#for: "selected_file",
                "Selected"
            }
            span {
                id: "selected_file",
                "{file_name}"
            }
        }
        button {
            style: "width: 100%",
            onclick: move |evt| {
                println!("Selected mode {}", selected_mode());
                output.set(start_deepfry(selected_mode(), red_value(), green_value(), blue_value(), &*file_path()));
            },
            "Deepfry!"
        }
        img {
            style: "width: 50%",
            src: "{output}"
        }
    )
}

fn cmp_enum<T>(this: &T, other: &T) -> bool {
    mem::discriminant(this) == mem::discriminant(other)
}

fn start_deepfry(mode: ChangeMode, red: u8, green: u8, blue: u8, path: &str) -> String {
    let mut image = image::open(path).unwrap().to_rgb8();

    deepfry(
        &mut image,
        BitChange(mode, red as u32, green as u32, blue as u32),
    ).unwrap();

    image_to_data_url(&image)
}

fn image_to_data_url(img: &RgbImage) -> String {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png).expect("Failed to write!");

    let b64 = BASE64_STANDARD.encode(buf.get_mut());

    format!("data:image/png;base64,{}", b64)
}