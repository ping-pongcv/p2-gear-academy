#![no_std]
#![allow(static_mut_refs)]

use game_session_io::*;
use gstd::{exec, msg};

const TRIES_LIMIT: u8 = 5;

static mut GAME_SESSION_STATE: Option<GameSession> = None;

#[no_mangle]
extern "C" fn init() {
    let game_session_init: GameSessionInit = msg::load().expect("Failed to decode GameSessionInit");
    game_session_init.assert_valid();

    unsafe {
        GAME_SESSION_STATE = Some(game_session_init.into());
    };
}

#[no_mangle]
extern "C" fn handle() {
    let game_session_action: GameSessionAction =
        msg::load().expect("Failed to decode GameSessionAction");
    let game_session = get_game_session_mut();
    match game_session_action {
        GameSessionAction::StartGame => {
            let user = msg::source();
            let session_info = game_session.sessions.entry(user).or_default();
            match &session_info.session_status {
                SessionStatus::ReplyReceived(wordle_event) => {
                    msg::reply::<GameSessionEvent>(wordle_event.into(), 0)
                        .expect("Failed to reply message");
                    session_info.session_status = SessionStatus::WaitUserInput;
                }
                SessionStatus::Init
                | SessionStatus::GameOver(..)
                | SessionStatus::WaitWordleStartReply => {
                    // Send "StartGame" message to Wordle program
                    let send_to_wordle_msg_id = msg::send(
                        game_session.wordle_program_id,
                        WordleAction::StartGame { user },
                        0,
                    )
                    .expect("Failed to send message");
                    session_info.session_id = msg::id(); // Save current message ID
                    session_info.original_msg_id = msg::id(); // Save initial message ID
                    session_info.send_to_wordle_msg_id = send_to_wordle_msg_id; // Save message ID sent to Wordle
                    session_info.tries = 0; // Initialize attempt count
                    session_info.session_status = SessionStatus::WaitWordleStartReply; // Update status to waiting for Wordle start reply

                    msg::send_delayed(
                        exec::program_id(),
                        GameSessionAction::CheckGameStatus {
                            user,
                            session_id: msg::id(),
                        },
                        0,
                        200,
                    )
                    .expect("Failed to send delayed message");
                    exec::wait(); // Wait for reply
                }
                SessionStatus::WaitUserInput | SessionStatus::WaitWordleCheckWordReply => {
                    panic!("User is already in game");
                }
            }
        }
        GameSessionAction::CheckWord { word } => {
            let user = msg::source();
            let session_info = game_session.sessions.entry(user).or_default();
            match &session_info.session_status {
                SessionStatus::ReplyReceived(wordle_event) => {
                    session_info.tries += 1; // Increment attempt count
                    if wordle_event.has_guessed() {
                        // If word is guessed correctly, game ends with victory status
                        session_info.session_status = SessionStatus::GameOver(GameStatus::Win);
                        msg::reply(GameSessionEvent::GameOver(GameStatus::Win), 0)
                            .expect("Failed to reply message");
                    } else if session_info.tries == TRIES_LIMIT {
                        // If attempt limit is reached, game ends with lose status
                        session_info.session_status = SessionStatus::GameOver(GameStatus::Lose);
                        msg::reply(GameSessionEvent::GameOver(GameStatus::Lose), 0)
                            .expect("Failed to reply message");
                    } else {
                        msg::reply::<GameSessionEvent>(wordle_event.into(), 0)
                            .expect("Failed to reply message");
                        session_info.session_status = SessionStatus::WaitUserInput;
                        // Update status to waiting for player input
                    }
                }
                SessionStatus::WaitUserInput | SessionStatus::WaitWordleCheckWordReply => {
                    // Verify submitted word is 5 letters long and all lowercase
                    assert!(
                        word.len() == 5 && word.chars().all(|c| c.is_lowercase()),
                        "Invalid word"
                    );
                    let send_to_wordle_msg_id = msg::send(
                        game_session.wordle_program_id,
                        WordleAction::CheckWord { user, word },
                        0,
                    )
                    .expect("Failed to send message");
                    session_info.original_msg_id = msg::id();
                    session_info.send_to_wordle_msg_id = send_to_wordle_msg_id;
                    session_info.session_status = SessionStatus::WaitWordleCheckWordReply; // Update status to waiting for Wordle check word reply
                    exec::wait(); // Wait for reply
                }
                SessionStatus::Init
                | SessionStatus::WaitWordleStartReply
                | SessionStatus::GameOver(..) => {
                    panic!("User is not in game");
                }
            }
        }
        GameSessionAction::CheckGameStatus { user, session_id } => {
            if msg::source() == exec::program_id() {
                if let Some(session_info) = game_session.sessions.get_mut(&user) {
                    if session_id == session_info.session_id
                        && !matches!(session_info.session_status, SessionStatus::GameOver(..))
                    {
                        session_info.session_status = SessionStatus::GameOver(GameStatus::Lose); // If time's up and not completed, game ends with lose status
                        msg::send(user, GameSessionEvent::GameOver(GameStatus::Lose), 0)
                            .expect("Failed to send message");
                    }
                }
            }
        }
    }
}

#[no_mangle]
extern "C" fn handle_reply() {
    let reply_to = msg::reply_to().expect("Failed to query reply_to data");
    let wordle_event: WordleEvent = msg::load().expect("Failed to decode WordleEvent");
    let game_session = get_game_session_mut();
    let user = wordle_event.get_user();
    if let Some(session_info) = game_session.sessions.get_mut(user) {
        if reply_to == session_info.send_to_wordle_msg_id && session_info.is_wait_reply_status() {
            session_info.session_status = SessionStatus::ReplyReceived(wordle_event); // Received reply from Wordle program
            exec::wake(session_info.original_msg_id).expect("Failed to wake message");
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let game_session = get_game_session();
    msg::reply::<GameSessionState>(game_session.into(), 0).expect("Failed to reply state query");
}

fn get_game_session_mut() -> &'static mut GameSession {
    unsafe {
        GAME_SESSION_STATE
            .as_mut()
            .expect("Game session not initialized")
    }
}
fn get_game_session() -> &'static GameSession {
    unsafe {
        GAME_SESSION_STATE
            .as_ref()
            .expect("Game session not initialized")
    }
}
