use models::{FieldInfo, SeriesInfo, Tag, ValueType};
use std::fs;

use super::*;

#[tokio::test]
async fn test_index_add_del() {
    let _ = fs::remove_dir_all("/tmp/index_test/db_test1");
    let mut index = db_index::DBIndex::from("/tmp/index_test/db_test1");

    let mut info1 = SeriesInfo::new(
        "db_test".to_string(),
        "table_test".to_string(),
        vec![Tag::new(b"host".to_vec(), b"h1".to_vec())],
        vec![
            FieldInfo::new(0, b"cpu".to_vec(), ValueType::Float),
            FieldInfo::new(0, b"mem".to_vec(), ValueType::Float),
        ],
    );
    let id = index.add_series_if_not_exists(&mut info1).unwrap();

    let key = index.get_series_key(id).unwrap().unwrap();
    assert_eq!(info1.tags(), key.tags());

    index.del_series_info(id).await.unwrap();
    let key = index.get_series_key(id).unwrap();
    assert_eq!(key, None);

    index.flush().await.unwrap();
}

#[tokio::test]
async fn test_index_id_list() {
    let _ = fs::remove_dir_all("/tmp/index_test/db_test2");
    let mut index = db_index::DBIndex::from("/tmp/index_test/db_test2");

    let mut info1 = SeriesInfo::new(
        "db_test".to_string(),
        "table_test".to_string(),
        vec![
            Tag::new(b"loc".to_vec(), b"bj".to_vec()),
            Tag::new(b"host".to_vec(), b"h1".to_vec()),
        ],
        vec![
            FieldInfo::new(0, b"cpu".to_vec(), ValueType::Float),
            FieldInfo::new(0, b"mem".to_vec(), ValueType::Float),
        ],
    );

    let mut info2 = SeriesInfo::new(
        "db_test".to_string(),
        "table_test".to_string(),
        vec![
            Tag::new(b"loc".to_vec(), b"bj".to_vec()),
            Tag::new(b"host".to_vec(), b"h2".to_vec()),
        ],
        vec![FieldInfo::new(0, b"mem".to_vec(), ValueType::Float)],
    );

    let mut info3 = SeriesInfo::new(
        "db_test".to_string(),
        "table_test".to_string(),
        vec![
            Tag::new(b"loc".to_vec(), b"bj".to_vec()),
            Tag::new(b"host".to_vec(), b"h3".to_vec()),
        ],
        vec![FieldInfo::new(0, b"mem".to_vec(), ValueType::Float)],
    );

    let id1 = index.add_series_if_not_exists(&mut info1).unwrap();
    let id2 = index.add_series_if_not_exists(&mut info2).unwrap();
    let id3 = index.add_series_if_not_exists(&mut info3).unwrap();

    let key1 = index.get_series_key(id1).unwrap().unwrap();
    let key2 = index.get_series_key(id2).unwrap().unwrap();

    let enc = utils::encode_inverted_index_key(
        &"table_test".to_string(),
        &"tag".as_bytes(),
        &"val".as_bytes(),
    );
    assert_eq!(enc, "table_test.tag=val".as_bytes());

    let tags = vec![
        Tag::new(b"loc".to_vec(), b"bj".to_vec()),
        Tag::new(b"host".to_vec(), b"h2".to_vec()),
    ];

    let list = index
        .get_series_id_list(&"table_test".to_string(), &tags)
        .await
        .unwrap();
    assert_eq!(vec![id2], list);

    let list = index
        .get_series_id_list(&"table_test".to_string(), &tags[0..0].to_vec())
        .await
        .unwrap();
    assert_eq!(vec![id1, id2, id3], list);

    index.flush().await.unwrap();
}

#[tokio::test]
async fn test_field_type() {
    let _ = fs::remove_dir_all("/tmp/index_test/db_test3");
    let mut index = db_index::DBIndex::from("/tmp/index_test/db_test3");

    let mut info1 = SeriesInfo::new(
        "db_test".to_string(),
        "table_test".to_string(),
        vec![
            Tag::new(b"loc".to_vec(), b"bj".to_vec()),
            Tag::new(b"host".to_vec(), b"h1".to_vec()),
        ],
        vec![
            FieldInfo::new(0, b"cpu".to_vec(), ValueType::Float),
            FieldInfo::new(0, b"mem".to_vec(), ValueType::Float),
        ],
    );

    let mut info2 = SeriesInfo::new(
        "db_test".to_string(),
        "table_test".to_string(),
        vec![
            Tag::new(b"loc".to_vec(), b"bj".to_vec()),
            Tag::new(b"host".to_vec(), b"h2".to_vec()),
        ],
        vec![FieldInfo::new(0, b"mem".to_vec(), ValueType::Unsigned)],
    );
    let id1 = index.add_series_if_not_exists(&mut info1).unwrap();
    let id2 = index.add_series_if_not_exists(&mut info2);

    let schema = index.get_table_schema(&"table_test".to_string());

    index.flush().await.unwrap();
}
