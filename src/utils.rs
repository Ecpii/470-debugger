use std::{
    fs::{self},
    io::Result,
};

pub fn save_watch_list(watch_list: &[String], name: &str) -> Result<()> {
    let write_data = watch_list.join("\n");
    let filename = format!("debugger_files/{name}.dbg_list");

    fs::write(&filename, write_data)?;

    Ok(())
}

pub fn load_watch_list(name: &str) -> Result<Vec<String>> {
    let filename = format!("debugger_files/{name}.dbg_list");

    let data = fs::read_to_string(&filename)?;

    let watch_list = data.split("\n").map(|x| x.to_owned()).collect();
    Ok(watch_list)
}
