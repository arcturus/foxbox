/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate serde_json;

use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Mutex;


type ConfigNameSpace = BTreeMap<String, String>;

type ConfigTree = BTreeMap<String, ConfigNameSpace>;

#[derive(Debug)]
pub struct ConfigStore {
    file_name: String,
    save_lock: Mutex<()>,
    config: ConfigTree
}

impl ConfigStore {
    pub fn new(file_name: &str) -> Self {
        ConfigStore {
            file_name: file_name.to_owned(),
            save_lock: Mutex::new(()),
            config: ConfigStore::load(file_name)
        }
    }

    pub fn set(&mut self, namespace: &str, property: &str, value: &str) {
        debug!("Setting config for {}::{} to {}", namespace, property, value);
        if !self.config.contains_key(namespace) {
            self.config.insert(namespace.to_owned(), ConfigNameSpace::new());
        }
        self.config.get_mut(namespace).unwrap().insert(property.to_owned(), value.to_owned());
        // TODO: Should be more intelligent than save on every write
        self.save();
    }

    pub fn get(&self, namespace: &str, property: &str) -> Option<&String> {
        if self.config.contains_key(namespace) {
            let res = self.config.get(namespace).unwrap().get(property);
            debug!("Config result for {}::{} is {:?}", namespace, property, res);
            res
        } else {
            debug!("No config result for {}::{}", namespace, property);
            None
        }
    }

    fn load(file_name: &str) -> ConfigTree {
        let empty_config = BTreeMap::new();
        let file = match File::open(&Path::new(file_name)) {
            Ok(file) => {
                file
            },
            Err(error) => {
                error!("Unable to open configuration file {}: {}",
                    file_name, error.to_string());
                return empty_config;
            }
        };
        let parsed_config: ConfigTree = match serde_json::from_reader(&file) {
            Ok(value) => value,
            Err(error) => {
                error!("Unable to generate JSON from config file {}: {}",
                    file_name, error.to_string());
                    empty_config
            }
        };

        debug!("Parsed config file: {:?}", parsed_config);
        parsed_config
    }

    fn save(&self) {
        let file_path = Path::new(&self.file_name);
        let mut update_name = self.file_name.clone();
        update_name.push_str(".updated");
        let update_path = Path::new(&update_name);

        let conf_as_json = serde_json::to_string_pretty(&self.config).unwrap();

        let _ = self.save_lock.lock().unwrap();
        match File::create(update_path)
            .map(|mut file| file.write_all(&conf_as_json.as_bytes()))
            .and_then(|_| { fs::copy(&update_path, &file_path) })
            .and_then(|_| { fs::remove_file(&update_path) }) {
                Ok(_) => debug!("Wrote configuration file {}", self.file_name),
                Err(error) => error!("While writing configuration file{}: {}",
                    self.file_name, error.to_string())
            };
    }
}


#[cfg(test)]
describe! config_store {

    before_each {
        use uuid::Uuid;
        use std::fs;
        let config_file_name = format!("conftest-{}.tmp", Uuid::new_v4().to_simple_string());
    }

    it "should remember properties" {
        // Block to make `config` go out of scope
        {
            let mut config = ConfigStore::new(&config_file_name);
            config.set("foo", "bar", "baz");
            assert_eq!(
                config.get("foo", "bar"),
                Some(&"baz".to_string())
            );
        }
        // Would use after_each, but after_each is never called:
        fs::remove_file(config_file_name).unwrap_or(());
    }

    it "should return None on non-existent namespaces" {
        // Block to make `config` go out of scope
        {
            let mut config = ConfigStore::new(&config_file_name);
            config.set("foo", "bar", "baz");
            assert_eq!(
                config.get("foofoo", "bar"),
                None
            );
        }
        fs::remove_file(config_file_name).unwrap_or(());
    }

    it "should return None on non-existent properties" {
        // Block to make `config` go out of scope
        {
            let mut config = ConfigStore::new(&config_file_name);
            config.set("foo", "bar", "baz");
            assert_eq!(
                config.get("foo", "barbar"),
                None
            );
        }
        fs::remove_file(config_file_name).unwrap_or(());
    }

    it "should remember things over restarts" {
        // Block to make `config` go out of scope
        {
            let mut config = ConfigStore::new(&config_file_name);
            config.set("foo", "bar", "baz");
        }
        // `config` should now be out of scope and dropped
        {
            let config = ConfigStore::new(&config_file_name);
            assert_eq!(
                config.get("foo", "bar"),
                Some(&"baz".to_string())
            );
        }
        fs::remove_file(config_file_name).unwrap_or(());
    }
}
