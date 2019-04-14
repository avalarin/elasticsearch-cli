use serde_json::Value;
use std::iter::FromIterator;
use std::convert::Into;
use std::collections::HashSet;
use std::collections::btree_map::BTreeMap;

pub struct JSONExtractor {
    field_delimiter: String,
    fields: Option<HashSet<String>>
}

impl JSONExtractor {
    pub fn default() -> Self {
        JSONExtractor {
            field_delimiter: ".".to_string(),
            fields: None
        }
    }

    pub fn filtered<S: Into<String>, I: IntoIterator<Item = S>>(fields: I) -> Self {
        JSONExtractor {
            field_delimiter: ".".to_string(),
            fields: Some(HashSet::from_iter(fields.into_iter().map(Into::into)))
        }
    }

    pub fn extract(&self, item: &Value) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        self.extract_one(&[], item, &mut map);
        map
    }

    fn extract_one(&self, path: &[String], hit: &Value, map: &mut BTreeMap<String, String>) {
        match hit {
            &Value::Object(ref object) => {
                for (key, value) in object {
                    let mut new_path = path.to_owned();
                    new_path.push(key.clone());
                    self.extract_one(&new_path, &value, map);
                }
            }
            &Value::Array(ref array) => {
                for (index, value) in array.iter().enumerate() {
                    let mut new_path = path.to_owned();
                    new_path.push(index.to_string());
                    self.extract_one(&new_path, &value, map);
                }
            }
            primitive => {
                let key = path.join(&self.field_delimiter);
                if self.is_field_ok(&key) {
                    map.insert(key, self.prepare_primitive(primitive));
                }
            }
        }
    }

    fn is_field_ok(&self, field: &str) -> bool {
        self.fields
            .as_ref()
            .map(|f| f.contains(field))
            .unwrap_or(true)
    }

    fn prepare_primitive(&self, value: &serde_json::Value) -> String {
        match value {
            &serde_json::Value::String(ref str_value) => str_value.to_string(),
            primitive => primitive.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::JSONExtractor;

    fn get_value() -> serde_json::Value {
        json!({
            "root": {
                 "obj": {
                      "strKey": "str1",
                      "intKey": 1
                 },
                 "arr": [
                    { "value": 1 },
                    { "value": 2 },
                    { "value": 3 }
                 ]
            }
        })
    }

    #[test]
    fn it_should_extract_values_from_json() {
        let map = JSONExtractor::default().extract(&get_value());
        assert_eq!(Some("str1".to_string()).as_ref(), map.get("root.obj.strKey"));
        assert_eq!(Some("1".to_string()).as_ref(),    map.get("root.obj.intKey"));
        assert_eq!(Some("1".to_string()).as_ref(),    map.get("root.arr.0.value"));
        assert_eq!(Some("2".to_string()).as_ref(),    map.get("root.arr.1.value"));
        assert_eq!(Some("3".to_string()).as_ref(),    map.get("root.arr.2.value"));

        assert_eq!(None, map.get("root"));
        assert_eq!(None, map.get("root.obj"));
        assert_eq!(None, map.get("root.obj.anotherKey"));
        assert_eq!(None, map.get("root.arr"));
        assert_eq!(None, map.get("root.arr.3.value"));
    }

    #[test]
    fn it_should_filter_fields() {
        let fields = vec!["root.arr.2.value", "root.obj.strKey"];
        let map = JSONExtractor::filtered(fields)
            .extract(&get_value());

        assert_eq!(Some("str1".to_string()).as_ref(), map.get("root.obj.strKey"));
        assert_eq!(Some("3".to_string()).as_ref(),    map.get("root.arr.2.value"));

        assert_eq!(None, map.get("root.obj.intKey"));
        assert_eq!(None, map.get("root.arr.0.value"));
        assert_eq!(None, map.get("root.arr.1.value"));
        assert_eq!(None, map.get("root"));
        assert_eq!(None, map.get("root.obj"));
        assert_eq!(None, map.get("root.obj.anotherKey"));
        assert_eq!(None, map.get("root.arr"));
        assert_eq!(None, map.get("root.arr.3.value"));
    }

}
