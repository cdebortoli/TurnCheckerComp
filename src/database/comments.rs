use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::models::{Comment, CommentType};

pub fn insert(connection: &Connection, comment: &Comment) -> Result<i64> {
    connection.execute(
        "INSERT INTO comments (comment_type, content) VALUES (?1, ?2)",
        params![comment.comment_type.as_str(), comment.content],
    )?;

    Ok(connection.last_insert_rowid())
}

pub fn fetch_all(connection: &Connection) -> Result<Vec<Comment>> {
    let mut statement =
        connection.prepare("SELECT id, comment_type, content FROM comments ORDER BY id")?;
    let rows = statement.query_map([], row_to_comment)?;

    let comments = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(comments)
}

pub fn fetch_by_id(connection: &Connection, id: i64) -> Result<Option<Comment>> {
    let mut statement =
        connection.prepare("SELECT id, comment_type, content FROM comments WHERE id = ?1")?;

    let comment = statement.query_row([id], row_to_comment).optional()?;
    Ok(comment)
}

pub fn update(connection: &Connection, comment: &Comment) -> Result<()> {
    connection.execute(
        "UPDATE comments SET comment_type = ?1, content = ?2 WHERE id = ?3",
        params![comment.comment_type.as_str(), comment.content, comment.id],
    )?;
    Ok(())
}

pub fn delete(connection: &Connection, id: i64) -> Result<()> {
    connection.execute("DELETE FROM comments WHERE id = ?1", [id])?;
    Ok(())
}

fn row_to_comment(row: &Row<'_>) -> rusqlite::Result<Comment> {
    let raw_type: String = row.get(1)?;
    Ok(Comment {
        id: row.get(0)?,
        comment_type: CommentType::from_str(&raw_type),
        content: row.get(2)?,
    })
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::database::connection::establish_in_memory_connection;
    use crate::models::{Comment, CommentType};

    #[test]
    fn comment_crud_round_trip() -> Result<()> {
        let connection = establish_in_memory_connection()?;
        let mut comment = Comment::new(CommentType::Turn, "Remember the timing");

        let id = super::insert(&connection, &comment)?;
        comment.id = id;

        let fetched = super::fetch_by_id(&connection, id)?.expect("comment exists");
        assert_eq!(fetched.comment_type, CommentType::Turn);

        comment.comment_type = CommentType::Game;
        comment.content = "Whole match note".to_string();
        super::update(&connection, &comment)?;

        let comments = super::fetch_all(&connection)?;
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].comment_type, CommentType::Game);

        super::delete(&connection, id)?;
        assert!(super::fetch_all(&connection)?.is_empty());

        Ok(())
    }
}
