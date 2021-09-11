# Othello AlphaZero

[near129/othello](https://github.com/near129/othello)のコンピュータAlphaZeroのNNのモデルを学習するレポジトリ

- `othello_alphazero/train_model.py` 学習するスクリプト
- `selfplay/main.rs` SelfPlayをして学習データをnumpyの保存形式で保存
- `selfplay/vs_random.rs` ランダムに石をおくコンピュータと対戦する

rustで書かれたコードは並列で処理する。
