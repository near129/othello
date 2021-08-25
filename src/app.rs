use othello::{board::{Stone, StoneCount}, game::Game, players::{Player, RandomPlayer}};
use yew::prelude::*;

pub struct App<> {
    game: Game,
    ai: RandomPlayer,
    link: ComponentLink<Self>,
}

pub enum Msg {
    Put(usize, usize),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            game: Game::default(),
            ai: RandomPlayer::new(Stone::Whilte),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Put(y, x) => {
                log::info!("put x: {} y: {}", x, y);
                if let Err(message) = self.game.put(x, y) {
                    log::error!("{}", message);
                } else if self.game.turn == Stone::Whilte {
                    if let Ok((x, y)) = self.ai.find_move(&self.game.board) {
                        if let Err(message) = self.game.put(x, y) {
                            log::error!("ai error: {}", message);
                        }
                        
                    }
                }
            }
        }
        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let StoneCount { black, white } = self.game.count_stone();
        let score_black = if self.game.turn == Stone::Black {
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
        let score_white = if self.game.turn == Stone::Whilte {
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
            .game
            .board
            .get_board()
            .iter()
            .flatten()
            .zip(self.game.get_available_squares().unwrap().iter().flatten())
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
                                (Some(Stone::Whilte), _) => {
                                    html! {
                                        <div class="piece">
                                            <div class="white"></div>
                                            <div class="black"></div>
                                        </div>
                                    }
                                }
                                (None, true) => {
                                    if self.game.turn == Stone::Black {
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
        html! {
            <div class="othello-game">
                <div class="score">
                    {vec![score_black, score_white]}
                </div>
                <div class="board">
                    { board }
                </div>
            </div>
        }
    }
}
