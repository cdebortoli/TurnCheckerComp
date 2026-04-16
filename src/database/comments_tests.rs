use anyhow::Result;

use crate::database::connection::establish_in_memory_connection;
use crate::models::{Comment, CommentType};

#[test]
fn comment_crud_round_trip() -> Result<()> {
    let connection = establish_in_memory_connection()?;
    let mut comment = Comment::new(CommentType::Turn, "Remember the timing");

    let id = super::insert(&connection, &comment)?;
    comment.id = id;

    let fetched = super::fetch_by_uuid(&connection, &comment.uuid)?.expect("comment exists");
    assert_eq!(fetched.comment_type, CommentType::Turn);
    assert!(!fetched.is_sent);

    comment.comment_type = CommentType::Game;
    comment.content = "Whole match note".to_string();
    comment.is_sent = true;
    assert_eq!(super::upsert(&connection, &comment)?, id);

    let comments = super::fetch_all(&connection)?;
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].comment_type, CommentType::Game);
    assert!(comments[0].is_sent);

    super::delete(&connection, id)?;
    assert!(super::fetch_all(&connection)?.is_empty());

    Ok(())
}
