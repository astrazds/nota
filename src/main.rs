mod app;
mod backup;
mod components;
mod storage;
mod ui;

fn main() {
    leptos::mount::mount_to_body(app::App);
}
