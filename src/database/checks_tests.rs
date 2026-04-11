use anyhow::Result;

use crate::database::connection::establish_in_memory_connection;
use crate::database::tags;
use crate::models::check_source_type::CheckSourceType;
use crate::models::{Check, CheckRepeatType, Tag};

#[test]
fn check_crud_round_trip() -> Result<()> {
    let connection = establish_in_memory_connection()?;
    let tag = Tag::new("Intel", "#112233", "#FFFFFF");
    tags::insert(&connection, &tag)?;
    let mut check = Check::new("Scout");
    check.detail = Some("Reveal nearby units".to_string());
    check.source = CheckSourceType::Blueprint;
    check.repeat_case = CheckRepeatType::Conditional(3);
    check.tag_uuid = Some(tag.uuid);
    check.position = 2;
    check.is_mandatory = true;
    check.is_checked = true;
    check.is_sent = true;

    let id = super::insert(&connection, &check)?;
    check.id = id;

    let fetched = super::fetch_by_uuid(&connection, &check.uuid)?.expect("check exists");
    assert_eq!(fetched.source, CheckSourceType::Blueprint);
    assert_eq!(fetched.repeat_case, CheckRepeatType::Conditional(3));
    assert_eq!(fetched.tag_uuid, Some(tag.uuid));
    assert!(fetched.is_sent);

    check.source = CheckSourceType::Turn;
    check.repeat_case = CheckRepeatType::Until(5);
    check.name = "Scout Again".to_string();
    super::update(&connection, &check)?;

    let checks = super::fetch_all(&connection)?;
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].name, "Scout Again");
    assert_eq!(checks[0].source, CheckSourceType::Turn);
    assert_eq!(checks[0].repeat_case, CheckRepeatType::Until(5));
    assert_eq!(checks[0].tag_uuid, Some(tag.uuid));

    super::delete(&connection, id)?;
    assert!(super::fetch_all(&connection)?.is_empty());

    Ok(())
}

#[test]
fn default_source_round_trips() -> Result<()> {
    let connection = establish_in_memory_connection()?;
    let check = Check::new("Default source");

    let id = super::insert(&connection, &check)?;
    let fetched = super::fetch_by_uuid(&connection, &check.uuid)?.expect("check exists");

    assert_eq!(fetched.id, id);
    assert_eq!(fetched.source, CheckSourceType::Game);

    Ok(())
}

#[test]
fn fetch_all_returns_only_global_game_and_turn_checks() -> Result<()> {
    let connection = establish_in_memory_connection()?;

    let mut global_check = Check::new("Global check");
    global_check.source = CheckSourceType::GlobalGame;

    let mut game_check = Check::new("Game check");
    game_check.source = CheckSourceType::Game;

    let mut turn_check = Check::new("Turn check");
    turn_check.source = CheckSourceType::Turn;

    super::insert(&connection, &global_check)?;
    super::insert(&connection, &game_check)?;
    super::insert(&connection, &turn_check)?;

    let mut fetched_global =
        super::fetch_by_uuid(&connection, &global_check.uuid)?.expect("global check exists");
    fetched_global.position = 99;
    super::update(&connection, &fetched_global)?;

    let mut fetched_game =
        super::fetch_by_uuid(&connection, &game_check.uuid)?.expect("game check exists");
    fetched_game.position = 2;
    super::update(&connection, &fetched_game)?;

    let mut fetched_turn =
        super::fetch_by_uuid(&connection, &turn_check.uuid)?.expect("turn check exists");
    fetched_turn.position = 1;
    super::update(&connection, &fetched_turn)?;

    let checks = super::fetch_all(&connection)?;

    assert_eq!(checks.len(), 2);
    assert_eq!(checks[0].source, CheckSourceType::GlobalGame);
    assert_eq!(checks[0].name, "Global check");
    assert_eq!(checks[1].source, CheckSourceType::Turn);
    assert_eq!(checks[1].name, "Turn check");

    Ok(())
}

#[test]
fn upsert_preserves_global_game_priority_in_fetch_all() -> Result<()> {
    let connection = establish_in_memory_connection()?;

    let mut global_check = Check::new("Global check");
    global_check.source = CheckSourceType::GlobalGame;
    super::insert(&connection, &global_check)?;

    let mut game_check = Check::new("Game check");
    game_check.source = CheckSourceType::Game;
    super::insert(&connection, &game_check)?;

    let mut fetched_global =
        super::fetch_by_uuid(&connection, &global_check.uuid)?.expect("global check exists");
    fetched_global.position = 99;
    fetched_global.is_sent = true;

    let mut fetched_game =
        super::fetch_by_uuid(&connection, &game_check.uuid)?.expect("game check exists");
    fetched_game.position = 1;
    super::update(&connection, &fetched_game)?;

    super::upsert(&connection, &fetched_global)?;

    let checks = super::fetch_all(&connection)?;

    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].source, CheckSourceType::GlobalGame);
    assert_eq!(checks[0].uuid, global_check.uuid);
    assert!(checks[0].is_sent);

    Ok(())
}

#[test]
fn fetch_by_source_returns_only_requested_source() -> Result<()> {
    let connection = establish_in_memory_connection()?;

    let mut game_check = Check::new("Game check");
    game_check.source = CheckSourceType::Game;

    let mut global_game_check = Check::new("Global game check");
    global_game_check.source = CheckSourceType::GlobalGame;

    let mut template_check = Check::new("Template check");
    template_check.source = CheckSourceType::Blueprint;

    super::insert(&connection, &game_check)?;
    super::insert(&connection, &global_game_check)?;
    super::insert(&connection, &template_check)?;

    let checks = super::fetch_by_source(&connection, CheckSourceType::Game)?;

    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].source, CheckSourceType::Game);
    assert_eq!(checks[0].name, "Game check");

    Ok(())
}

#[test]
fn count_unsent_returns_only_unsent_checks() -> Result<()> {
    let connection = establish_in_memory_connection()?;

    let unsent = Check::new("Unsent");
    super::insert(&connection, &unsent)?;

    let mut sent = Check::new("Sent");
    sent.is_sent = true;
    super::insert(&connection, &sent)?;

    assert_eq!(super::count_unsent(&connection)?, 1);

    Ok(())
}
