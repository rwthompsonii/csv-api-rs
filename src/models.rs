use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Record {
    pub id_str: String, //make the assumption this is globally unique
    pub a_int: i32,
    pub opt_str: Option<String>,
    pub opt_float: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct JsonError {
    pub err: String,
}

#[cfg(test)]
mod test {
    use std::f64::consts::{E, PI};

    use super::*;

    //NOTE: I just used this test to get a nice easy serialized string that I could then deserialize
    #[test]
    pub fn test() {
        let record = Record {
            id_str: "some_string".to_string(),
            a_int: 67890,
            opt_str: Some("my cool string".to_string()),
            opt_float: Some(PI),
        };
        let record2 = Record {
            id_str: "another-string".to_string(),
            a_int: 8675309,
            opt_str: None,
            opt_float: None,
        };
        let record3 = Record {
            id_str: "yet-another-string".to_string(),
            a_int: -3478347,
            opt_str: None,
            opt_float: Some(E),
        };
        let record4 = Record {
            id_str: "totally-amazing-string".to_string(),
            a_int: 3748343,
            opt_str: Some("yay".to_string()),
            opt_float: None,
        };
        let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
        writer.serialize(record).unwrap();
        writer.serialize(record2).unwrap();
        writer.serialize(record3).unwrap();
        writer.serialize(record4).unwrap();
        println!(
            "{}",
            String::from_utf8_lossy(&*writer.into_inner().unwrap())
        );
    }
}
