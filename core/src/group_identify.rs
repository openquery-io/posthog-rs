use chrono::NaiveDateTime;
use serde::Serialize;
use std::collections::HashMap;

use crate::{
    error::Error,
    event::{Event, Properties},
};

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct GroupIdentify {
    group_type: String,
    group_key: String,
    group_properties: Option<Properties>,
    timestamp: Option<NaiveDateTime>,
}

impl GroupIdentify {
    pub fn new<S: Into<String>>(group_type: S, group_id: S) -> Self {
        Self {
            group_type: group_type.into(),
            group_key: group_id.into(),
            group_properties: None,
            timestamp: None,
        }
    }

    pub fn insert_prop<K: Into<String>, P: Serialize>(
        &mut self,
        key: K,
        prop: P,
    ) -> Result<(), Error> {
        let as_json =
            serde_json::to_value(prop).map_err(|source| Error::Serialization { source })?;
        let _ = self
            .group_properties
            .as_mut()
            .unwrap()
            .props
            .insert(key.into(), as_json);
        Ok(())
    }
}

impl TryFrom<GroupIdentify> for Event {
    type Error = Error;
    fn try_from(group_identify: GroupIdentify) -> Result<Self, Self::Error> {
        let distinct_id = format!("{}_{}", group_identify.group_type, group_identify.group_key);

        let mut props: HashMap<String, serde_json::Value> = HashMap::with_capacity(3);
        props.insert("$group_type".into(), group_identify.group_type.into());
        props.insert("$group_key".into(), group_identify.group_key.into());
        if let Some(properties) = group_identify.group_properties {
            props.insert("$group_set".into(), serde_json::to_value(properties.props)?);
        }

        Ok(Self {
            event: "$groupidentify".into(),
            properties: Properties {
                distinct_id,
                props,
                groups: None,
            },
            timestamp: group_identify.timestamp,
        })
    }
}