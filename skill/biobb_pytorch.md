---
name: biobb_pytorch
category: deep-learning / bioinformatics
description: BioBB PyTorch utilities for training and inferring molecular properties using deep neural networks within the BioExcel Building Blocks ecosystem. Provides Python-wrapped CLI and programmatic interfaces for loading molecular datasets, training GNN or MLP models on structural features, and exporting trained weights for downstream simulation or analysis pipelines.
tags:
  - deep-learning
  - pytorch
  - molecular-dynamics
  - bioinformatics
  - neural-networks
  - gnn
  - biobb
  - structure-prediction
  - data-loader
  - model-export
author: AI-generated
source_url: https://github.com/bioexcel/biobb_pytorch
---

## Concepts

- **Dataset Format — NumPy `.npz` archives or PDB-level feature vectors**: input molecular descriptors (atom types, bond angles, radii, charges) are packed into `input_features.npz` files with named arrays `X` (features) and `y` (targets). Passing an incorrectly keyed `.npz` archive causes a silent shape mismatch that corrupts gradient flow during training.
- **Model Architecture — Graph Neural Network (GNN) or Feed-Forward MLP via PyTorch `nn.Module`**: the tool exposes `GNNModel` and `MLPModel` classes whose forward pass expects a fixed-batch feature tensor. Instantiating the wrong class for your data modality leads to a dimension error at the first `forward()` call.
- **Checkpointing — Stateful `.pt` / `.pth` weight files**: after each training epoch, a checkpoint dictionary containing model weights, optimizer state, and the current epoch index is saved. Loading a checkpoint whose architecture definition is missing or mismatched causes a key-error crash when resuming training.
- **Device Abstraction — CPU / CUDA toggle**: the tool auto-detects GPU availability via `torch.cuda.is_available()`. If CUDA is present but the DataLoader returns CPU tensors, moving data to the device inside the training loop is the developer's responsibility; a device mismatch silently sends the loss to NaN.
- **CLI Wrapper — Python entry point via `biobb_pytorch train / eval / predict`**: subcommands expose train, evaluate, and predict modes. Mixing subcommand flags (e.g., passing `--model-architecture` to `predict`) raises an `ArgumentError` because architecture flags are trainer-only.

## Pitfalls

- **Mismatched feature dimensionality between dataset and declared input size**: if the `.npz` `X` array has shape `(N, 128)` but the model expects `(N, 256)` input features, PyTorch throws a `RuntimeError: size mismatch`. Always verify the dataset descriptor file matches the declared `--input-dim` argument before launching training.
- **Using a CPU-only checkpoint on a CUDA-enabled host without device migration**: loading a `.pt` checkpoint saved on CPU into a model that is `.to("cuda")` results in a type error because tensor device layout is serialized. Explicitly call `model.load_state_dict(torch.load(path, map_location="cuda"))` to rebind tensors to the correct device.
- **Failing to normalize target values before training regression models**: when `y` contains raw energies in kcal/mol (e.g., spanning −500 to +500), the large magnitude causes gradient explosion and NaN loss after a few epochs. Always apply Z-score or Min-Max normalization to `y` and pass the scaler parameters to the `--target-mean` / `--target-std` flags.
- **Oversubscribing GPU memory by setting `--batch-size` too large**: a batch size exceeding available VRAM causes `torch.cuda.OutOfMemoryError`. BioBB PyTorch does not auto-split batches; reduce `--batch-size` in increments of 4 until the error disappears, and monitor with `nvidia-smi`.
- **Ignoring the `--num-workers` DataLoader setting on Windows or macOS**: setting `--num-workers > 0` on non-Linux platforms without guarding the main module with `if __name__ == "__main__"` causes fork-related crashes. BioBB PyTorch prints a warning but still attempts multiprocessing, leading to a silent hang.

## Examples

### Train a GNN model on molecular descriptors with default hyperparameters
**Args:** `train --input-npz ./data/train_descriptors.npz --input-dim 128 --hidden-dim 256 --output-dim 1 --epochs 50 --lr 0.001 --batch-size 16 --checkpoint-dir ./checkpoints/`
**Explanation:** Loads the NumPy archive, builds a two-layer GNN with 128 input features and a 256-unit hidden layer, trains for 50 epochs using Adam at a learning rate of 0.001, and saves a checkpoint after each epoch to the specified directory.

### Evaluate a pre-trained model and report RMSE on a held-out test set
**Args:** `eval --checkpoint-path ./checkpoints/model_epoch040.pt --test-npz ./data/test_descriptors.npz --input-dim 128 --output-dim 1 --batch-size 32`
**Explanation:** Resumes the saved model weights from epoch 40, runs forward inference on the test split, and prints the root-mean-square error between predicted and true target values to stderr.

### Predict molecular binding affinity for new PDB-derived features
**Args:** `predict --checkpoint-path ./checkpoints/model_epoch050.pt --input-npz ./data/new_ligand_features.npz --input-dim 128 --output-dim 1 --device cuda --output-csv ./predictions.csv`
**Explanation:** Loads a trained model into GPU memory, feeds newly computed feature vectors through the forward pass, writes the scalar affinity predictions to a CSV file, and exits cleanly.

### Resume interrupted training from the last checkpoint with a reduced learning rate
**Args:** `train --checkpoint-dir ./checkpoints/ --resume --input-npz ./data/train_descriptors.npz --input-dim 128 --hidden-dim 256 --output-dim 1 --epochs 80 --lr 0.0001 --batch-size 16`
**Explanation:** Scans the checkpoint directory, identifies the highest epoch checkpoint, reloads model and optimizer state, and continues training from epoch 40 with a tenfold lower learning rate to avoid oscillatory convergence.

### Export trained model weights to ONNX for portable inference in external workflows
**Args:** `export-onnx --checkpoint-path ./checkpoints/model_epoch050.pt --output-onnx ./model.onnx --input-dim 128 --batch-size 1 --input-name "features" --output-name "affinity"`
**Explanation:** Serializes the trained PyTorch `nn.Module` into ONNX format with a dummy batch dimension, enabling deployment in non-Python runtime environments such as REST APIs or mobile applications.

### Train an MLP model on flattened structural fingerprints with early stopping
**Args:** `train --model-type MLP --input-npz ./data/fingerprints.npz --input-dim 512 --hidden-dim 128 --output-dim 1 --epochs 100 --patience 5 --batch-size 64 --checkpoint-dir ./mlp_checkpoints/`
**Explanation:** Switches to a feed-forward MLP architecture instead of the default GNN, trains on high-dimensional fingerprint vectors, and halts training automatically if validation loss does not improve for 5 consecutive epochs, saving the best-performing snapshot.

### Generate per-atom gradient saliency maps using a trained model and visualize
**Args:** `predict --checkpoint-path ./checkpoints/model_epoch050.pt --input-npz ./data/complex_features.npz --input-dim 128 --output-dim 1 --saliency --saliency-output ./saliency_maps/`
**Explanation:** Runs a forward pass followed by backpropagation with respect to the output neuron to produce a gradient-based saliency heatmap per atom, and writes NumPy arrays per structural complex for downstream visualization with PyMOL or similar tools.