use std::io::{BufWriter, Write, ErrorKind, Read};
use std::fs::{read_dir, create_dir, exists, File};

use log::{error, info, debug};
use json::{array, stringify, parse};

pub trait Saver {
    fn save(&self, data: &String);
}

fn rewrite_exists_file(data: &String, full_path: &String) {
    debug!("File already exists!");
    let mut file_ = File::options().write(true).open(full_path.as_str()).unwrap();

    let mut json_str = String::new();
    if let Err(error) = file_.read_to_string(&mut json_str) {
        error!("Error when saving to file: {}", error);
    }

    let parsed_json_obj = parse(&json_str);

    if let Ok(mut json_obj) = parsed_json_obj {
        let parsed_input_json = parse(data);

        if let Ok(input_json_obj) = parsed_input_json {
            let result_str: String;
            if json_obj.is_array() {
                if let Err(push_err) = json_obj.push(input_json_obj) {
                    error!("Push error: {}", push_err);
                }

                result_str = stringify(json_obj);
            }
            else {
                let array_of_exists_json = array![json_obj, input_json_obj];

                result_str = stringify(array_of_exists_json);
            }

            if let Err(error) = file_.write_all(result_str.as_bytes()) {
                error!("Error in rewrite: {}", error);
            }
        }
        else if let Err(convert_error) = parsed_input_json {
            error!("Convert error: {}", convert_error);
        }
    }
    else if let Err(convert_error) = parsed_json_obj {
        error!("Convert error: {}", convert_error);
    }
}

pub struct FileSaver {
    file_path: String,
}

impl FileSaver {
    pub fn new(path: String) -> FileSaver {
        return FileSaver { file_path: path };
    }

    pub fn get_file_path(&self) -> &String {
        return &self.file_path;
    }
}

impl Saver for FileSaver {
    fn save(&self, data: &String) {
        info!("Saving data to file!");

        let full_path: String = self.file_path.clone();

        if let Err(error_when_create_dir) = create_dir(full_path.clone()) {
            if error_when_create_dir.kind() != ErrorKind::AlreadyExists {
                return;
            }
        }

        if let Ok(is_exists) = exists(&full_path) {
            if is_exists {
                rewrite_exists_file(data, &full_path);
            }
        }
        else {
            let new_file_result: Result<File, std::io::Error> = File::create(full_path);

            if let Ok(mut file_) = new_file_result {
                if let Err(error_in_write_op) = file_.write_all(data.as_bytes()) {
                    error!("Error in write operation. In new file: {}", error_in_write_op);
                }
            }
            else if let Err(error) = new_file_result {
                error!("Error when creating file: {}", error);
            }
        }
    }
}