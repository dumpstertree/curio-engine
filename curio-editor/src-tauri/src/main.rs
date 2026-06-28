// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use curio_core::io::file::File;
use curio_editor_lib::{Project, PROJECT};
use std::sync::Mutex;

pub struct Services {}

fn main() {
    //load project
    let local_project = serde_yaml::from_slice::<Project>(&File::read("./test.proj"));
    let Ok(project) = local_project else {
        panic!("No project found in local directory");
    };

    println!("PROJECT LOADED: {}, {}", project.name, project.project_path);
    // load project into memory
    unsafe { PROJECT = Some(Mutex::new(project)) };

    //
    curio_editor_lib::run()
}
