from pathlib import Path
import sys
import numpy as np
from sklearn.model_selection import train_test_split
import torch
from torch import nn
from torch.utils.data import DataLoader
import pytorch_lightning as pl
from pytorch_lightning.callbacks import ModelCheckpoint, EarlyStopping


class Model(torch.nn.Module):
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
        self.softmax = nn.LogSoftmax(dim=1)
        self.tanh = nn.Tanh()

    def forward(self, x):
        x = self.layer1(x)
        x = x.view(-1, self.fc_input)
        x = self.layer2(x)
        policy = self.fc3(x)
        value = self.fc4(x)
        return self.softmax(policy), self.tanh(value)


class Dataset:
    def __init__(self, states, policy, values) -> None:
        self.states = states
        self.policy = policy
        self.values = values

    def __len__(self):
        return len(self.values)

    def __getitem__(self, idx):
        return self.states[idx], self.policy[idx], self.values[idx]


class LightingModule(pl.LightningModule):
    def __init__(self) -> None:
        super().__init__()
        self.model = Model()
        self.loss_p = nn.BCEWithLogitsLoss()
        self.loss_v = nn.MSELoss()

    def forward(self, x):
        return self.model(x)

    def shared_step(self, batch):
        x, policy, value = batch
        p, v = self.model(x)
        return self.loss_p(policy, p) + self.loss_v(value, v)

    def training_step(self, batch, batch_idx):
        loss = self.shared_step(batch)
        self.log('train_loss', loss)
        return loss

    def validation_step(self, batch, batch_idx):
        loss = self.shared_step(batch)
        self.log('val_loss', loss)

    def configure_optimizers(self):
        return torch.optim.SGD(self.parameters(), lr=0.01)


def main():
    data_path = Path(sys.argv[1])
    policy = np.load(data_path / 'policy.npy')
    states = np.load(data_path / 'states.npy').astype(np.float32)
    values = np.load(data_path / 'values.npy').astype(np.float32)
    print(policy.shape, states.shape, values.shape)
    train_p, val_p, train_s, val_s, train_v, val_v = train_test_split(
        policy, states, values, test_size=0.2, shuffle=True, random_state=42
    )
    train_dataset = Dataset(train_s, train_p, train_v)
    val_dataset = Dataset(val_s, val_p, val_v)
    train_dataloder = DataLoader(train_dataset, batch_size=256, shuffle=True, num_workers=1)
    val_dataloder = DataLoader(val_dataset, batch_size=256, num_workers=1)
    module = LightingModule()
    trainer = pl.Trainer(
        callbacks=[
            ModelCheckpoint(monitor='val_loss'),
            EarlyStopping(monitor='val_loss'),
        ]
    )
    trainer.fit(module, train_dataloder, val_dataloder)
    model = module.model
    model.eval()
    dummy_input = torch.randn(1, 2, 8, 8)
    torch.onnx.export(model, dummy_input, "models/model.onnx")


if __name__ == '__main__':
    main()
