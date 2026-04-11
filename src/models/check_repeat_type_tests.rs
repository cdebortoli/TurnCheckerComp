use super::CheckRepeatType;

#[test]
fn repeat_type_to_storage() {
    assert_eq!(CheckRepeatType::Everytime.to_storage(), ("everytime", None));
    assert_eq!(
        CheckRepeatType::Conditional(3).to_storage(),
        ("conditional", Some(3))
    );
    assert_eq!(
        CheckRepeatType::Specific(7).to_storage(),
        ("specific", Some(7))
    );
    assert_eq!(CheckRepeatType::Until(4).to_storage(), ("until", Some(4)));
}

#[test]
fn repeat_type_from_storage() {
    assert_eq!(
        CheckRepeatType::from_storage("conditional", Some(3)),
        CheckRepeatType::Conditional(3)
    );
    assert_eq!(
        CheckRepeatType::from_storage("specific", Some(7)),
        CheckRepeatType::Specific(7)
    );
    assert_eq!(
        CheckRepeatType::from_storage("until", Some(4)),
        CheckRepeatType::Until(4)
    );
    assert_eq!(
        CheckRepeatType::from_storage("unknown", None),
        CheckRepeatType::Everytime
    );
}
