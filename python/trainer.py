import torch
from torch.utils.data import Dataset, DataLoader
from torch import optim, nn, tensor
import csv


class DriveDataset(Dataset):
    def __init__(self, csv_file):
        self.rows = []
        with open(csv_file) as csvfile:
            reader = csv.DictReader(csvfile)
            for row in reader:
                self.rows.append(row)
        
    def __len__(self):
        return len(self.rows)

    def __getitem__(self, idx):
        row = self.rows[idx]
        inpu = tensor([float(row['offset_error']), float(row['angle_error'])])
        output = tensor([float(row['angle']), float(row['speed'])])
        return inpu, output

class NeuralNetwork(nn.Module):
    def __init__(self):
        super().__init__()
        self.linear_relu_stack = nn.Sequential(
            nn.Linear(2, 10),
            nn.ReLU(),
            nn.Linear(10, 10),
            nn.ReLU(),
            nn.Linear(10, 2),
            nn.Sigmoid()
        )

    def forward(self, x):
        
        logits = self.linear_relu_stack(x)
        return logits

train_dataloader = DataLoader(DriveDataset("/home/donunt/picar/logs/2023-08-10 20:52:05.514518446 UTC"), batch_size=1, shuffle=True)
test_dataloader = DataLoader(DriveDataset("/home/donunt/picar/logs/2023-08-10 20:49:09.931862964 UTC"), batch_size=1, shuffle=True)

model = NeuralNetwork()

learning_rate = 1e-3
batch_size = 64
epochs = 5
loss_fn = nn.MSELoss()
optimizer = optim.Adam(model.parameters())
def train_loop(dataloader, model, loss_fn, optimizer):
    size = len(dataloader.dataset)
    # Set the model to training mode - important for batch normalization and dropout layers
    # Unnecessary in this situation but added for best practices
    model.train()
    for batch, (X, y) in enumerate(dataloader):
        # Compute prediction and loss
        pred = model(X)
        loss = loss_fn(pred, y)

        # Backpropagation
        loss.backward()
        optimizer.step()
        optimizer.zero_grad()

        if batch % 100 == 0:
            loss, current = loss.item(), (batch + 1) * len(X)
            print(f"loss: {loss:>7f}  [{current:>5d}/{size:>5d}]")
def test_loop(dataloader, model, loss_fn):
    # Set the model to evaluation mode - important for batch normalization and dropout layers
    # Unnecessary in this situation but added for best practices
    model.eval()
    size = len(dataloader.dataset)
    num_batches = len(dataloader)
    test_loss, correct = 0, 0

    # Evaluating the model with torch.no_grad() ensures that no gradients are computed during test mode
    # also serves to reduce unnecessary gradient computations and memory usage for tensors with requires_grad=True
    with torch.no_grad():
        for X, y in dataloader:
            pred = model(X)
            test_loss += loss_fn(pred, y).item()
            correct += (pred.argmax(1) == y).type(torch.float).sum().item()

    test_loss /= num_batches
    correct /= size
    print(f"Test Error: \n Accuracy: {(100*correct):>0.1f}%, Avg loss: {test_loss:>8f} \n")
train_loop(train_dataloader, model, loss_fn, optimizer)
test_loop(test_dataloader, model, loss_fn)
torch.onnx.export(model, tensor([0.3, 0.5]), "driver.onnx", verbose=True, input_names=["offset", "angle"])
print(tensor([0.0, 0.0]).shape)