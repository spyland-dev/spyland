/*
 *  spyland-lib — public library API for accessing spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

//! Module `config` is a small utility module to comfortably manage multiple sections in one main
//! spyland configuration file. Use this module if you build an official spyland software or
//! a compositor backend.

use anyhow::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::PathBuf;
use toml::Value;

/// A really simple trait that represents your section in spyland's configuration. Use it as your
/// (de)serializable configuration. It requires implementing
/// [`Serialize`], [`DeserializeOwned`], and [`Default`].
pub trait ConfigSection: Serialize + DeserializeOwned + Default {
    /// That constant represents your section's name. Typically
    /// you should use `backend.compositor-name`. If you build something different,
    /// consider using your own configuration.
    const SECTION: &'static str;
}

/// A wrapper for the main spyland configuration file, used for convenient handling of multiple sections.
pub struct ConfigFile {
    value: toml::Value,
    path: PathBuf,
}

impl ConfigFile {
    /// Creates a new instance of [`ConfigFile`].
    ///
    /// # Arguments
    /// - `path` --- target config path
    pub fn new(path: PathBuf) -> Result<Self> {
        let value = toml::from_str(&fs::read_to_string(&path)?)?;
        Ok(Self { path, value })
    }

    /// Updates the current value by reading the config file.
    pub fn load(&mut self) -> Result<()> {
        self.value = toml::from_str(&fs::read_to_string(&self.path)?)?;

        Ok(())
    }

    /// Saves the current value into the config file.
    pub fn save(&self) -> Result<()> {
        fs::write(&self.path, toml::to_string(&self.value)?)?;

        Ok(())
    }

    /// Gets the [section](ConfigSection) from a config.
    pub fn get_section<T>(&self) -> Result<T>
    where
        T: ConfigSection,
    {
        let mut value = &self.value;

        for part in T::SECTION.split('.') {
            match value.get(part) {
                Some(section) => value = section,
                None => return Ok(T::default()),
            }
        }

        Ok(value.clone().try_into()?)
    }

    /// Overwrites the [section](ConfigSection) with the one you provide.
    pub fn set_section<T>(&mut self, section: T) -> Result<()>
    where
        T: ConfigSection,
    {
        let value = Value::try_from(section)?;

        let mut current = &mut self.value;

        let mut parts = T::SECTION.split('.').peekable();

        while let Some(part) = parts.next() {
            if parts.peek().is_some() {
                current = current
                    .as_table_mut()
                    .ok_or_else(|| {
                        anyhow::anyhow!("Config section '{}' is not a table", T::SECTION)
                    })?
                    .entry(part)
                    .or_insert_with(|| Value::Table(Default::default()));
            } else {
                current
                    .as_table_mut()
                    .ok_or_else(|| {
                        anyhow::anyhow!("Parent section of '{}' is not a table", T::SECTION)
                    })?
                    .insert(part.to_string(), value.clone());
            }
        }

        Ok(())
    }
}
