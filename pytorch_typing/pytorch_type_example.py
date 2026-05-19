import torch

def process_tensor(x: torch.Tensor) -> torch.Tensor:
    return x * 2

x = torch.tensor([1.0, 2.0, 3.0])
print(f"Processed tensor: {process_tensor(x)}")