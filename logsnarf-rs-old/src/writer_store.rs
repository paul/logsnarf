use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::settings::Settings;
use crate::writer::Writer;
use crate::{credentials, credentials::CredentialsError};

pub struct WriterStore<'a> {
    settings: &'a Settings,
    credentials: &'a mut credentials::Store<'a>,
    writers: Writers<'a>,
}

// TODO replace HashMap with a TTL Cache
type Writers<'a> = HashMap<&'a String, Result<&'a mut Writer, CredentialsError>>;

impl<'a> WriterStore<'_> {
    pub fn new(settings: &'a Settings) -> WriterStore<'a> {
        let credentials = &mut credentials::Store::new(&settings);
        WriterStore {
            settings,
            credentials,
            writers: Writers::with_capacity(100),
        }
    }

    pub fn get(&mut self, token: &String) -> Result<&'a mut Writer, CredentialsError> {
        match self.writers.get_mut(token) {
            Some(res) => *res,
            None => {
                let creds = self.credentials.fetch(token)?;
                let writer = Writer::new(creds);
                self.writers.insert(token, Ok(&mut writer));
                Ok(&mut writer)
            }
        }
    }

    pub fn flush_all(&self) {}
}
