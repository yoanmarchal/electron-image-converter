// Pas de console derrière la fenêtre dans la version distribuée
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> iced::Result {
    image_converter::ui::run()
}
