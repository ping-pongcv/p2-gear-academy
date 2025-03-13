#[cfg(test)]
mod tests {
    use gstd::{prelude::*, msg, ActorId};
    use gtest::{Program, System};
    use game_session_io::*;

    const WORDLE_ID: u64 = 100;
    const GAME_SESSION_ID: u64 = 200;
    const USER1: u64 = 10;



    #[test]
    fn test_start_game() {
        let sys = System::new();
        sys.init_logger();

        let wordle = Program::from_file(&sys, "../target/wasm32-unknown-unknown/debug/wordle.wasm");
        let game_session = Program::from_file(
            &sys,
            "../target/wasm32-unknown-unknown/debug/game_session.wasm",
        );

        let user_id: ActorId = USER1.into();
        let wordle_id: ActorId = WORDLE_ID.into();
        assert!(!wordle.send(user_id, wordle_id).main_failed());
        assert!(!game_session.send(user_id, wordle_id).main_failed());

        let game_session = sys.get_program(GAME_SESSION_ID).unwrap();

        assert!(!game_session.send(USER1, GameSessionAction::StartGame).main_failed());

        let state: GameSessionState = game_session.read_state(()).unwrap();
        assert!(state.game_sessions.iter().any(|(user, _)| *user == USER1.into()));
        println!("as: {:?}", state);

        let session_info = &state
            .game_sessions
            .iter()
            .find(|(user, _)| *user == USER1.into())
            .unwrap()
            .1;
        assert!(matches!(
            session_info.session_status,
            SessionStatus::WaitUserInput
        ));
    }

    #[test]
    fn test_check_word_correct_check_win() {
        let sys = System::new();
        sys.init_logger();

        let wordle = Program::from_file(&sys, "../target/wasm32-unknown-unknown/debug/wordle.wasm");
        let game_session = Program::from_file(
            &sys,
            "../target/wasm32-unknown-unknown/debug/game_session.wasm",
        );

        let user_id: ActorId = USER1.into();
        let wordle_id: ActorId = WORDLE_ID.into();
        assert!(!wordle.send(user_id, wordle_id).main_failed());
        assert!(!game_session.send(user_id, wordle_id).main_failed());

        let game_session = sys.get_program(GAME_SESSION_ID).unwrap();

        // 模拟用户发送 StartGame 请求
        assert!(!game_session.send(USER1, GameSessionAction::StartGame).main_failed());
        let state: GameSessionState = game_session.read_state(()).unwrap();
        println!("111: {:?}", state);

        // 模拟用户发送 CheckWord 请求
        assert!(!game_session.send(USER1, GameSessionAction::CheckWord { word: "hello".to_string() }).main_failed());
        assert!(!game_session.send(USER1, GameSessionAction::CheckWord { word: "hello".to_string() }).main_failed());
        // 检查会话状态是否为 ReplyReceived
        let state: GameSessionState = game_session.read_state(()).unwrap();
        let session_info = &state
            .game_sessions
            .iter()
            .find(|(user, _)| *user == USER1.into())
            .unwrap()
            .1;
        println!("as: {:?}", state);

        // 检查尝试次数是否增加
        assert_eq!(session_info.tries, 2);

        // 模拟用户再次发送 CheckWord 请求
        assert!(!game_session.send(USER1, GameSessionAction::CheckWord { word: "house".to_string() }).main_failed());

        // 检查尝试次数是否更新为2
        let state: GameSessionState = game_session.read_state(()).unwrap();
        let session_info = &state
            .game_sessions
            .iter()
            .find(|(user, _)| *user == USER1.into())
            .unwrap()
            .1;
        assert_eq!(session_info.tries, 3);
        assert!(matches!(
            session_info.session_status,
            SessionStatus::GameOver(_)
        ));
    }

    #[test]
    fn test_game_over() {
        let sys = System::new();
        sys.init_logger();

        let wordle = Program::from_file(&sys, "../target/wasm32-unknown-unknown/debug/wordle.wasm");
        let game_session = Program::from_file(
            &sys,
            "../target/wasm32-unknown-unknown/debug/game_session.wasm",
        );

        let user_id: ActorId = USER1.into();
        let wordle_id: ActorId = WORDLE_ID.into();
        assert!(!wordle.send(user_id, wordle_id).main_failed());
        assert!(!game_session.send(user_id, wordle_id).main_failed());
        let game_session = sys.get_program(GAME_SESSION_ID).unwrap();

        // 模拟用户发送 StartGame 请求
        assert!(!game_session.send(USER1, GameSessionAction::StartGame).main_failed());

        // 模拟用户发送 CheckWord 请求，直到游戏结束
        assert!(!game_session.send(USER1, GameSessionAction::CheckWord { word: "hello".to_string() }).main_failed());
        assert!(!game_session.send(USER1, GameSessionAction::CheckWord { word: "wrong".to_string() }).main_failed());
        assert!(!game_session.send(USER1, GameSessionAction::CheckWord { word: "wrong".to_string() }).main_failed());
        assert!(!game_session.send(USER1, GameSessionAction::CheckWord { word: "wrong".to_string() }).main_failed());
        assert!(!game_session.send(USER1, GameSessionAction::CheckWord { word: "wrong".to_string() }).main_failed());
        // assert!(!game_session.send(USER1, GameSessionAction::CheckWord { word: "wrong".to_string() }).main_failed());

        // 检查会话状态是否为 GameOver
        let state: GameSessionState = game_session.read_state(()).unwrap();
        let session_info = &state
            .game_sessions
            .iter()
            .find(|(user, _)| *user == USER1.into())
            .unwrap()
            .1;
        assert!(matches!(session_info.session_status, SessionStatus::GameOver(_)));
    }

    #[test]
    fn test_time(){
        let sys = System::new();
        sys.init_logger();

        let wordle = Program::from_file(&sys, "../target/wasm32-unknown-unknown/debug/wordle.wasm");
        let game_session = Program::from_file(
            &sys,
            "../target/wasm32-unknown-unknown/debug/game_session.wasm",
        );

        let user_id: ActorId = USER1.into();
        let wordle_id: ActorId = WORDLE_ID.into();
        assert!(!wordle.send(user_id, wordle_id).main_failed());
        assert!(!game_session.send(user_id, wordle_id).main_failed());
        let game_session = sys.get_program(GAME_SESSION_ID).unwrap();

        assert!(!game_session.send(USER1, GameSessionAction::StartGame).main_failed());

        sys.spend_blocks(200);

        let state: GameSessionState = game_session.read_state(()).unwrap();
        let session_info = &state
            .game_sessions
            .iter()
            .find(|(user, _)| *user == USER1.into())
            .unwrap()
            .1;
        println!("as: {:?}", state);

        assert!(matches!(session_info.session_status, SessionStatus::GameOver(_)));
    }
}

