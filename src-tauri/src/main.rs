// Não abrir a janela de console no Windows em release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    jarvas_desktop_lib::run()
}
