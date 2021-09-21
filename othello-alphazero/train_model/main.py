import os
import shutil
import subprocess
import warnings
from pathlib import Path

import numpy as np
import pytorch_lightning as pl
import timm
import torch
import torch.multiprocessing
import torch.utils.data
import typer
from pytorch_lightning.callbacks import EarlyStopping
from sklearn.model_selection import train_test_split
from torch import nn
from torch.utils.data import DataLoader
import matplotlib.pyplot as plt

warnings.simplefilter('ignore')


class SimpleModel(torch.nn.Module):
    def __init__(self):
        super().__init__()
        self.height = self.width = 8
        self.ouput_size = 8 * 8
        self.dropout_late = 0.5
        in_channels = 2
        channels = 16

        self.relu = nn.ReLU()
        self.layer1 = nn.Sequential(
            nn.Conv2d(in_channels, channels, 3, padding=1),
            nn.BatchNorm2d(channels),
            self.relu,
            nn.Conv2d(channels, channels, 3, padding=1),
            nn.BatchNorm2d(channels),
            self.relu,
            nn.Conv2d(channels, channels, 3),
            nn.BatchNorm2d(channels),
            self.relu,
            nn.Conv2d(channels, channels, 3),
            nn.BatchNorm2d(channels),
            self.relu,
        )

        self.fc_input = channels * (self.width - 4) * (self.height - 4)
        self.dropout = nn.Dropout(self.dropout_late, inplace=True)
        self.layer2 = nn.Sequential(
            nn.Linear(self.fc_input, 512),
            nn.BatchNorm1d(512),
            self.relu,
            self.dropout,
            nn.Linear(512, 256),
            nn.BatchNorm1d(256),
            self.relu,
            self.dropout,
        )

        self.fc3 = nn.Linear(256, 64)
        self.fc4 = nn.Linear(256, 1)
        self.softmax = nn.Softmax(dim=1)
        self.tanh = nn.Tanh()

    def forward(self, x):
        x = self.layer1(x)
        x = x.view(-1, self.fc_input)
        x = self.layer2(x)
        policy = self.fc3(x)
        value = self.fc4(x)
        return self.softmax(policy), self.tanh(value)


class Model(torch.nn.Module):
    def __init__(self, backbone='regnetx_002', features=256, dropout=0.3):
        super().__init__()
        self.features = features
        self.dropout = dropout
        self.backbone = timm.create_model(
            backbone, num_classes=self.features, in_chans=2
        )

        self.bn = nn.BatchNorm1d(self.features)
        self.relu = nn.ReLU()
        self.dropout = nn.Dropout(self.dropout, inplace=True)

        self.fc3 = nn.Linear(256, 64)
        self.fc4 = nn.Linear(256, 1)
        self.softmax = nn.Softmax(dim=1)
        self.tanh = nn.Tanh()

    def forward(self, x):
        x = self.backbone(x)
        x = self.dropout(self.relu(self.bn(x)))
        policy = self.fc3(x)
        value = self.fc4(x)
        return self.softmax(policy), self.tanh(value)


class Dataset(torch.utils.data.Dataset):
    def __init__(self, states, policy, values) -> None:
        self.states = states
        self.policy = policy
        self.values = values

    def __len__(self):
        return len(self.values)

    def __getitem__(self, idx):
        return self.states[idx], self.policy[idx], self.values[idx]


class LightingModule(pl.LightningModule):
    def __init__(self, model='simple') -> None:
        super().__init__()
        self.save_hyperparameters()
        self.model = SimpleModel() if model == 'simple' else Model(model)
        self.loss_p = nn.BCEWithLogitsLoss()
        self.loss_v = nn.MSELoss()

    def forward(self, x):
        return self.model(x)

    def shared_step(self, batch):
        x, policy, value = batch
        p, v = self.model(x)
        return self.loss_p(policy, p) + self.loss_v(value, v.squeeze())

    def training_step(self, batch, batch_idx):
        loss = self.shared_step(batch)
        self.log('train_loss', loss)
        return loss

    def validation_step(self, batch, batch_idx):
        loss = self.shared_step(batch)
        self.log('val_loss', loss)

    def configure_optimizers(self):
        return torch.optim.Adam(self.parameters(), lr=0.001)


def main(
    initial_training=True,
    model_path: Path = Path('models/latest.ckpt'),
    onnx_model_path: Path = Path('models/model.onnx'),
    data_path: Path = Path('data'),
    num_simulation: int = 500,
    num_iter: int = 100,
    num_worker=os.cpu_count(),
    data_augment = True,
    batch_size=64,
    model='simple',
):
    subprocess.run(
        [
            'cargo',
            'build',
            '--release',
        ]
    ).check_returncode()
    module = (
        LightingModule(model)
        if initial_training
        else LightingModule.load_from_checkpoint(model_path)
    )
    result = []
    for i in range(num_iter):
        print(f'**********{i}************')
        if data_path.exists():
            shutil.rmtree(data_path)
            os.mkdir(data_path)
        subprocess.run(
            [
                '../target/release/selfplay',
                'data',
                str(num_worker),
                str(num_simulation),
            ]
        ).check_returncode()
        policy = np.load(data_path / 'policy.npy')
        states = np.load(data_path / 'states.npy').astype(np.float32)
        values = np.load(data_path / 'values.npy').astype(np.float32)
        if data_augment:
            policy = [np.rot90(policy.reshape(-1, 8, 8), i, (1, 2)).reshape(-1, 64) for i in range(4)]
            states = [np.rot90(states, i, (2, 3)) for i in range(4)]
            values = [values for _ in range(4)] * 2
            policy += [np.fliplr(p) for p in policy]
            states += [np.fliplr(s) for s in states]
            policy = np.concatenate(policy)
            states = np.concatenate(states)
            values = np.concatenate(values)
        train_p, val_p, train_s, val_s, train_v, val_v = train_test_split(
            policy, states, values, test_size=0.2, shuffle=True, random_state=42
        )
        train_dataset = Dataset(train_s, train_p, train_v)
        val_dataset = Dataset(val_s, val_p, val_v)
        train_dataloder = DataLoader(train_dataset, batch_size=batch_size, shuffle=True, drop_last=True)
        val_dataloder = DataLoader(val_dataset, batch_size=batch_size, drop_last=True)
        trainer = pl.Trainer(
            max_epochs=20,
            log_every_n_steps=10,
            logger=[],
            callbacks=[
                EarlyStopping(monitor='val_loss'),
            ],
            checkpoint_callback=False,
            gpus=-1 if torch.cuda.is_available() else 0
        )
        trainer.fit(module, train_dataloder, val_dataloder)
        trainer.save_checkpoint(model_path)
        if i % 3 == 2:
            res = subprocess.run(
                [
                    '../target/release/vs_random',
                    '50',
                ],
                capture_output=True
            )
            res.check_returncode()
            result.append(float(res.stdout.decode()))
            plt.plot(result)
            plt.savefig(model_path.parent / 'result.png')
        dummy_input = torch.randn(1, 2, 8, 8)
        module.to_onnx(onnx_model_path, dummy_input, export_params=True)


if __name__ == '__main__':
    typer.run(main)
