use std::time::Duration;

use othello::{
    players::{AlphaZeroPlayer, Player},
    Board, Position, Stone, StoneCount,
};
use yew::{
    prelude::*,
    services::{timeout::TimeoutTask, TimeoutService},
};

pub enum State {
    Loading(TimeoutTask),
    Passed(TimeoutTask),
    PlayerTurn,
    GameOver,
    GameSetting,
}
pub struct App {
    board: Board,
    ai: AlphaZeroPlayer,
    link: ComponentLink<Self>,
    state: State,
    player_stone: Stone,
}

pub enum Msg {
    Put(usize, usize),
    PutComputer,
    SelectPlayer(Stone),
    Restart,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            board: Board::default(),
            ai: AlphaZeroPlayer::new(300),
            link,
            state: State::GameSetting,
            player_stone: Stone::Black,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Put(y, x) => {
                let pos = Position::new(x, y);
                if let Err(message) = self.board.put(pos) {
                    log::error!("{}", message);
                } else if self.board.turn != self.player_stone {
                    let handle = TimeoutService::spawn(
                        Duration::from_millis(1),
                        self.link.callback(|_| Msg::PutComputer),
                    );
                    self.state = State::Loading(handle);
                } else if self.board.finished() {
                    self.state = State::GameOver;
                }
            }
            Msg::PutComputer => {
                match self.ai.find_move(&self.board) {
                    Ok(pos) => {
                        if let Err(message) = self.board.put(pos) {
                            log::error!("ai error: {}", message);
                        }
                        self.state = State::PlayerTurn;
                    }
                    Err(message) => {
                        log::error!("{}", message);
                    }
                };
                if self.board.turn != self.player_stone {
                    let handle = TimeoutService::spawn(
                        Duration::from_millis(1),
                        self.link.callback(|_| Msg::PutComputer),
                    );
                    self.state = State::Passed(handle);
                }
                if self.board.finished() {
                    self.state = State::GameOver;
                }
            }
            Msg::SelectPlayer(stone) => {
                self.player_stone = stone;
                if self.board.turn == self.player_stone {
                    self.state = State::PlayerTurn;
                } else {
                    self.link.send_message(Msg::PutComputer);
                }
            }
            Msg::Restart => {
                self.board.init();
                self.state = State::GameSetting;
            }
        }
        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let StoneCount { black, white } = self.board.count_stone();
        let score_black = if self.board.turn == Stone::Black {
            html! {
                <div class="points outlined">
                    <div class="demopiece black"></div><span>{black}</span>
                </div>
            }
        } else {
            html! {
                <div class="points">
                    <div class="demopiece black"></div><span>{black}</span>
                </div>
            }
        };
        let score_white = if self.board.turn == Stone::White {
            html! {
                <div class="points outlined">
                    <div class="demopiece white"></div><span>{white}</span>
                </div>
            }
        } else {
            html! {
                <div class="points">
                    <div class="demopiece white"></div><span>{white}</span>
                </div>
            }
        };
        let board = self
            .board
            .create_board_array()
            .iter()
            .flatten()
            .zip(self.board.get_legal_moves().to_map().iter().flatten())
            .enumerate()
            .map(|(idx, square)| {
                let click_callback = self.link.callback(move |_| Msg::Put(idx / 8, idx % 8));
                html! {
                    <div class="box" onclick=click_callback>
                        {
                            match square {
                                (Some(Stone::Black), _) => {
                                    html! {
                                        <div class="piece back">
                                            <div class="white"></div>
                                            <div class="black"></div>
                                        </div>
                                    }
                                }
                                (Some(Stone::White), _) => {
                                    html! {
                                        <div class="piece">
                                            <div class="white"></div>
                                            <div class="black"></div>
                                        </div>
                                    }
                                }
                                (None, true) => {
                                    if self.board.turn == Stone::Black {
                                        html! {
                                            <div class="avail avail-black"></div>
                                        }
                                    } else {
                                        html! {
                                            <div class="avail"></div>
                                        }
                                    }
                                }
                                (None, false) => {
                                    html! {}
                                }
                            }
                        }
                    </div>
                }
            })
            .collect::<Vec<Html>>();
        let state = html! {
            match self.state {
                State::GameOver => {
                    html! {
                        <div class="statecontainer">
                            <div class="statecard">
                                <span class="result">
                                    {
                                        if black == white { "Drow" } else if (
                                            self.player_stone == Stone::Black && black > white)
                                            || (self.player_stone == Stone::White && black < white)
                                            { "Win" } else { "Lose" }
                                    }
                                </span>
                                <span class="again" onclick=self.link.callback(|_| Msg::Restart)>{ "Play again" }</span>
                            </div>
                        </div>
                    }
                },
                State::Loading(_) => {
                    html! {
                        <div class="statecontainer">
                            <div class="statecard">{ "Searching..." }</div>
                        </div>
                    }
                },
                State::Passed(_) => {
                    html! {
                        <div class="statecontainer">
                            <div class="statecard">{ "Pass" }</div>
                        </div>
                    }
                },
                State::PlayerTurn => {
                    html! {}
                },
                State::GameSetting => {
                    html! {
                        <div class="statecontainer">
                            <div class="statecard">
                                <span class="selectplayer">{ "Select player" }</span>
                                <div class="player">
                                    <div class="firstplayer", onclick=self.link.callback(|_| Msg::SelectPlayer(Stone::Black))>
                                        { "First Player" }
                                    </div>
                                    <div class="secondplayer", onclick=self.link.callback(|_| Msg::SelectPlayer(Stone::White))>
                                        { "Second Player" }
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                },
            }
        };
        html! {
            <div class="othello-game">
                <div class="score">
                    {vec![score_black, score_white]}
                </div>
                <div class="board">
                    { board }
                </div>
                    { state }
            </div>
        }
    }
}
