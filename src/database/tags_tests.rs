use anyhow::Result;

use crate::database::connection::establish_in_memory_connection;
use crate::models::Tag;

#[test]
fn tag_crud_round_trip() -> Result<()> {
    let connection = establish_in_memory_connection()?;
    let mut tag = Tag::new("Attack", "#FF0000", "#FFFFFF");

    let id = super::insert(&connection, &tag)?;
    tag.id = id;

    let fetched = super::fetch_by_uuid(&connection, &tag.uuid)?.expect("tag exists");
    assert_eq!(fetched.name, "Attack");

    tag.name = "Defense".to_string();
    assert_eq!(super::upsert(&connection, &tag)?, id);

    let tags = super::fetch_all(&connection)?;
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "Defense");

    super::delete(&connection, id)?;
    assert!(super::fetch_all(&connection)?.is_empty());

    Ok(())
}
