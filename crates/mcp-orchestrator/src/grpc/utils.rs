use std::fmt::Debug;

use crate::{podmcp::McpPodError, storage::label_query::LabelQuery};
use prost_wkt_types::Any;
use proto::mcp::orchestrator::v1::{self, LabelKeyValue, LabelKeyValues};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tonic::Status;

pub fn convert_label_query(label: v1::LabelQuery) -> Vec<LabelQuery> {
    let mut queries = Vec::new();
    for LabelKeyValue { key, value } in label.equal {
        queries.push(LabelQuery::Equal { key, value });
    }
    for LabelKeyValue { key, value } in label.not_equal {
        queries.push(LabelQuery::NotEqual { key, value });
    }
    for key in label.contain_key {
        queries.push(LabelQuery::ContainKey { key });
    }
    for key in label.not_contain_key {
        queries.push(LabelQuery::NotContainKey { key });
    }
    for LabelKeyValues { key, values } in label.r#in {
        queries.push(LabelQuery::In { key, values });
    }
    for LabelKeyValues { key, values } in label.not_in {
        queries.push(LabelQuery::NotIn { key, values });
    }
    queries
}

pub fn convert_from_any<D: DeserializeOwned>(value: &Any) -> Result<D, Status> {
    let s = serde_json::from_slice(&value.value)
        .map_err(|e| Status::invalid_argument(format!("Failed to deserialize Any: {}", e)))?;
    Ok(s)
}

pub fn convert_to_any<S: Serialize + ?Sized>(value: &S) -> Result<Any, Status> {
    let type_id = std::any::type_name::<S>();

    let bytes = serde_json::to_vec(value)
        .map_err(|e| Status::invalid_argument(format!("Failed to serialize to Any: {}", e)))?;
    Ok(Any {
        type_url: type_id.to_string(),
        value: bytes,
    })
}

// pub fn to_wkt_time(dt: chrono::DateTime<chrono::Utc>) -> prost_wkt_types::Timestamp {
//     prost_wkt_types::Timestamp {
//         seconds: dt.timestamp(),
//         nanos: dt.timestamp_subsec_nanos() as i32,
//     }
// }

// pub fn to_wkt_time(dt: chrono::DateTime<chrono::Utc>) -> prost_wkt_types::Timestamp {
//     prost_wkt_types::Timestamp {
//         seconds: dt.timestamp(),
//         nanos: dt.timestamp_subsec_nanos() as i32,
//     }
// }

pub trait ProtoWktTime {
    fn to_wkt_time(&self) -> prost_wkt_types::Timestamp;
}

impl<Tz: chrono::TimeZone> ProtoWktTime for chrono::DateTime<Tz> {
    fn to_wkt_time(&self) -> prost_wkt_types::Timestamp {
        let tx = self.with_timezone(&chrono::Utc);
        prost_wkt_types::Timestamp {
            seconds: tx.timestamp(),
            nanos: tx.timestamp_subsec_nanos() as i32,
        }
    }
}

impl ProtoWktTime for chrono::NaiveDateTime {
    fn to_wkt_time(&self) -> prost_wkt_types::Timestamp {
        let dt = self.and_utc();
        prost_wkt_types::Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }
}

impl ProtoWktTime for k8s_openapi::apimachinery::pkg::apis::meta::v1::Time {
    fn to_wkt_time(&self) -> prost_wkt_types::Timestamp {
        let dt = self.0.clone();
        dt.to_wkt_time()
    }
}
