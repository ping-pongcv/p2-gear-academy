use game_session_io::*;
use gtest::{Log, ProgramBuilder, System};

const GAME_SESSION_PROGRAM_ID: u64 = 1;
const WORDLE_PROGRAM_ID: u64 = 2;
const USER: u64 = 50; 

#[test]
fn test_win() {
    let system = System::new();
    system.init_logger();

    let game_session_program = ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/game_session.opt.wasm")
        .with_id(GAME_SESSION_PROGRAM_ID)
        .build(&system);
    let wordle_program = ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/wordle.opt.wasm")
        .with_id(WORDLE_PROGRAM_ID)
        .build(&system);

    let res = wordle_program.send_bytes(USER, []);
    assert!(!res.main_failed());

    let res = game_session_program.send(
        USER,
        GameSessionInit {
            wordle_program_id: WORDLE_PROGRAM_ID.into(),
        },
    );
    assert!(!res.main_failed());

    let res = game_session_program.send(
        USER,
        GameSessionAction::CheckWord {
            word: "abcde".to_string(),
        },
    );
    assert!(res.main_failed());

    let res = game_session_program.send(USER, GameSessionAction::StartGame);
    let log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::StartSuccess);
    assert!(!res.main_failed() && res.contains(&log));

    let res = game_session_program.send(USER, GameSessionAction::StartGame);
    assert!(res.main_failed());

    let res = game_session_program.send(
        USER,
        GameSessionAction::CheckWord {
            word: "Abcde".to_string(),
        },
    );
    assert!(res.main_failed());

    let res = game_session_program.send(
        USER,
        GameSessionAction::CheckWord {
            word: "abcdef".to_string(),
        },
    );
    assert!(res.main_failed());

    let res = game_session_program.send(
        USER,
        GameSessionAction::CheckWord {
            word: "house".to_string(),
        },
    );
    let log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::CheckWordResult {
            correct_positions: vec![0, 1, 3, 4],
            contained_in_word: vec![],
        });
    assert!(!res.main_failed() && res.contains(&log));

    let res = game_session_program.send(
        USER,
        GameSessionAction::CheckWord {
            word: "horse".to_string(),
        },
    );
    let log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::GameOver(GameStatus::Win));
    assert!(!res.main_failed() && res.contains(&log));

    let res = game_session_program.send(
        51,
        GameSessionAction::CheckWord {
            word: "abcde".to_string(),
        },
    );
    assert!(res.main_failed());

    let state: GameSessionState = game_session_program.read_state(b"").unwrap();
    println!("{:?}", state);
}

#[test]
fn test_tried_limit() {
    let system = System::new();
    system.init_logger();

    let game_session_program = ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/game_session.opt.wasm")
        .with_id(GAME_SESSION_PROGRAM_ID)
        .build(&system);
    let wordle_program = ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/wordle.opt.wasm")
        .with_id(WORDLE_PROGRAM_ID)
        .build(&system);

    let res = wordle_program.send_bytes(USER, []);
    assert!(!res.main_failed());

    let res = game_session_program.send(
        USER,
        GameSessionInit {
            wordle_program_id: WORDLE_PROGRAM_ID.into(),
        },
    );
    assert!(!res.main_failed());

    let res = game_session_program.send(USER, GameSessionAction::StartGame);
    let log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::StartSuccess);
    assert!(!res.main_failed() && res.contains(&log));

    for i in 0..5 {
        let res = game_session_program.send(
            USER,
            GameSessionAction::CheckWord {
                word: "house".to_string(),
            },
        );
        if i == 4 {
            let log = Log::builder()
                .dest(USER)
                .source(GAME_SESSION_PROGRAM_ID)
                .payload(GameSessionEvent::GameOver(GameStatus::Lose));
            assert!(!res.main_failed() && res.contains(&log));
        } else {
            let log = Log::builder()
                .dest(USER)
                .source(GAME_SESSION_PROGRAM_ID)
                .payload(GameSessionEvent::CheckWordResult {
                    correct_positions: vec![0, 1, 3, 4],
                    contained_in_word: vec![],
                });
            assert!(!res.main_failed() && res.contains(&log));
        }
    }
    let state: GameSessionState = game_session_program.read_state(b"").unwrap();
    println!("{:?}", state);
}

#[test]
fn test_delayed_logic() {
    let system = System::new();
    system.init_logger();

    let game_session_program = ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/game_session.opt.wasm")
        .with_id(GAME_SESSION_PROGRAM_ID)
        .build(&system);
    let wordle_program = ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/wordle.opt.wasm")
        .with_id(WORDLE_PROGRAM_ID)
        .build(&system);

    let res = wordle_program.send_bytes(USER, []);
    assert!(!res.main_failed());

    let res = game_session_program.send(
        USER,
        GameSessionInit {
            wordle_program_id: WORDLE_PROGRAM_ID.into(),
        },
    );
    assert!(!res.main_failed());

    let res = game_session_program.send(USER, GameSessionAction::StartGame);
    let log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::StartSuccess);
    assert!(!res.main_failed() && res.contains(&log));

    let result = system.spend_blocks(200);
    println!("{:?}", result);

    let log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::GameOver(GameStatus::Lose));
    assert!(result[0].contains(&log));

    let state: GameSessionState = game_session_program.read_state(b"").unwrap();
    println!("{:?}", state);
}
