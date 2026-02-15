use chrono::Utc;
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};
use sanctum_core::{App, Avatar, Emote, InputMode, RaceType, ui};

#[test]
fn test_visual_delete_confirmation_integrity() {
    let mut avatar = Avatar::summon();
    avatar.name = "TestHero".to_string();
    let mut app = App::new(avatar);
    app.input_mode = InputMode::ConfirmingDelete("TestHero".to_string());

    let backend = TestBackend::new(80, 25);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            ui(f, &mut app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let content = buffer_to_string(buffer);

    assert!(
        content.contains("Are you certain you wish to delete TestHero?"),
        "Heading missing"
    );
    assert!(content.contains("(y) Yes, Delete"), "Delete option missing");
    assert!(content.contains("(n) No, Cancel"), "Cancel option missing");
}

#[test]
fn test_hero_rank_canonicalization() {
    let test_cases = vec![
        (RaceType::Orc, "Grunt"),
        (RaceType::Human, "Footman"),
        (RaceType::Undead, "Ghoul"),
        (RaceType::NightElf, "Archer"),
    ];

    for (race, expected_rank) in test_cases {
        let mut avatar = Avatar::summon();
        avatar.race_type = race;
        avatar.xp = 0;
        assert_eq!(
            avatar.rank(),
            expected_rank,
            "Race {race:?} must start as {expected_rank}"
        );
    }
}

#[test]
fn test_emote_coverage_and_wit_integrity() {
    let heroes = vec![
        "Thrall",
        "Grom",
        "Cairne",
        "Vol'jin",
        "Rexxar",
        "Rokhan",
        "Arthas",
        "Uther",
        "Jaina",
        "Muradin",
        "Antonidas",
        "Kael'thas",
        "Sylvanas",
        "Kel'Thuzad",
        "Anub'arak",
        "Mal'Ganis",
        "Varimathras",
        "Illidan",
        "Tyrande",
        "Malfurion",
        "Maiev",
        "Akama",
        "Lady Vashj",
        "Chen",
    ];
    let emote_options = vec![
        Emote::Cheer,
        Emote::Roar,
        Emote::Dance,
        Emote::Salute,
        Emote::Ponder,
        Emote::Flex,
    ];

    for name in heroes {
        let mut avatar = Avatar::summon();
        avatar.name = name.to_string();

        for emote in &emote_options {
            let msg = avatar.get_lore_emote_message(*emote);
            assert!(
                !msg.is_empty(),
                "Hero {name} has empty message for emote {emote:?}"
            );
            assert!(msg.len() > 5, "Hero {name} message too short for {emote:?}");
        }
    }
}

#[test]
fn test_race_fallback_emote_integrity() {
    let races = vec![
        RaceType::Orc,
        RaceType::Human,
        RaceType::Undead,
        RaceType::NightElf,
    ];
    let emote_options = vec![
        Emote::Cheer,
        Emote::Roar,
        Emote::Dance,
        Emote::Salute,
        Emote::Ponder,
        Emote::Flex,
    ];

    for race in races {
        let mut avatar = Avatar::summon();
        avatar.race_type = race;
        avatar.name = "UnknownUnit".to_string(); // Trigger fallback

        for emote in &emote_options {
            let msg = avatar.get_lore_emote_message(*emote);
            // Fallbacks might be empty for some combos, but we check critical ones
            if matches!(emote, Emote::Salute | Emote::Ponder | Emote::Roar) {
                assert!(
                    !msg.is_empty(),
                    "Race {race:?} missing fallback for {emote:?}"
                );
            }
        }
    }
}

fn buffer_to_string(buffer: &Buffer) -> String {
    let mut s = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            s.push_str(buffer.get(x, y).symbol());
        }
        s.push('\n');
    }
    s
}

#[test]
fn test_objective_persistence_switch_integrity() {
    let mut hero_a = Avatar::summon();
    hero_a.add_task("Objective A".to_string());
    let state_a = hero_a.to_state();

    let restored_a = Avatar::from_state(state_a);
    assert_eq!(
        restored_a.tasks.len(),
        1,
        "Objectives MUST survive reincarnation"
    );
}

#[test]
fn test_xp_multipliers_stacking() {
    let mut avatar = Avatar::summon();
    avatar.ultimate_active_until = Some(Utc::now() + chrono::Duration::minutes(1));
    avatar.rested_xp = 100;
    let old_xp = avatar.xp;
    avatar.link_contribution("Proj".to_string(), "Msg".to_string());
    let gain = avatar.xp - old_xp;
    assert!(gain >= 45);
}
