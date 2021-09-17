from sys import argv

import torch
from main import LightingModule

def main():
    lightningmodule = LightingModule.load_from_checkpoint(argv[1])
    dummy_input = torch.randn(1, 2, 8, 8)
    lightningmodule.to_onnx(argv[2], dummy_input, export_params=True)
    
    
if __name__ == '__main__':
    main()