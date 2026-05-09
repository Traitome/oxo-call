---
name: biobb_ml
category: bioinformatics/machine_learning
description: A bioinformatics building block providing machine learning capabilities for molecular biology data analysis, including clustering, classification, regression, and dimensionality reduction using popular ML libraries.
tags: machine-learning, bioinformatics, clustering, classification, regression, dimensionality-reduction, scikit-learn, tensorflow, keras
author: AI-generated
source_url: https://github.com/bioexcel/biobb_ml
---

## Concepts

- **Input data formats**: biobb_ml accepts tabular data in CSV or TSV format where columns represent features and rows represent samples. The tool expects a defined feature column (using `--input_data_path` or `--features_col`) and optionally a target column (using `--target_col`) for supervised learning tasks.
- **Output artifacts**: Depending on the task, biobb_ml generates different outputs including prediction tables (CSV/JSON), trained model files (joblib or pickle), and visualization plots (PNG/PS). Dimensionality reduction tasks output transformed coordinate datasets, while clustering produces cluster assignment labels.
- **Subcommands for ML tasks**: The tool is organized by functional subcommands representing different algorithms (e.g., `kmeans`, `pca`, `randomforest`, `svc`, `tsne` for clustering, dimensionality reduction, classification respectively). Each subcommand has its own specific parameters controlling algorithm behavior, hyperparameters, and output options.
- **Integration with ML libraries**: biobb_ml wraps popular machine learning libraries including scikit-learn (for classical ML algorithms), TensorFlow, and Keras (for deep learning). This enables consistent data handling and pipeline integration while leveraging well-validated implementations.

## Pitfalls

- **Mismatched feature columns**: Specifying a column name in `--features_col` that does not exist in the input dataset causes the task to fail silently or produce meaningless results. Always verify column names in your input file before running.
- **Missing target column for supervised learning**: When running classification or regression tasks without specifying `--target_col`, the tool may either error out or produce a model that cannot be evaluated. Ensure the target column exists and contains valid labels for your task.
- **Insufficient data for training**: Using too few samples relative to the number of features causes severe overfitting. biobb_ml does not automatically validate model generalizability—always use cross-validation or a separate test set to assess performance.
- **Incompatible output directory**: Specifying an `--output_json_path` or `--results_path` to a directory that lacks write permissions results in a permission error. Verify directory permissions before execution.

## Examples

### Perform K-means clustering on molecular descriptor data

**Args:** `--input_data_path=descriptors.csv --input_sep=, --output_json_path=cluster_results.json --output_csv_path=clustered_data.csv --n_clusters=5 --features_col=desc_1,desc_2,desc_3,desc_4,desc_5 --random_state=42`

**Explanation:** This runs K-means clustering with 5 clusters on molecular descriptors, using the specified feature columns and a fixed random seed for reproducibility. The clustered assignments are saved to the output CSV.

### Reduce dimensionality using PCA on gene expression matrix

**Args:** `--input_data_path=expression.csv --input_sep=\t --output_json_path=pca_results.json --output_csv_path=pca_coordinates.csv --features_col=gene_1:gene_100 --n_components=10`

**Explanation:** Principal Component Analysis reduces the 100 gene features to 10 principal components, useful for visualization or as input to downstream machine learning tasks. The transformed coordinates are written to the output file.

### Train a Random Forest classifier on protein functional classes

**Args:** `--input_data_path=training_data.csv --input_sep=, --output_model_path=rf_model.pkl --output_json_path=rf_results.json --features_col=feat_1:feat_50 --target_col=class_label --n_estimators=100 --test_size=0.2 --random_state=123`

**Explanation:** Trains a Random Forest classifier with 100 trees to predict protein functional classes, using 80% of data for training and 20% for testing. The model is serialized to a pickle file for later prediction.

### Apply t-SNE for visualizing single-cell RNA-seq data

**Args:** `--input_data_path=scRNA_matrix.csv --input_sep=\t --output_png_path=tsne_plot.png --features_col=col_1:col_1000 --n_components=2 --perplexity=30 --learning_rate=200 --n_iter=1000`

**Explanation:** t-SNE projects high-dimensional single-cell expression data into 2D for visualization, with perplexity controlling the balance between local and global structure preservation.

### Train a Support Vector Classifier for ligand activity prediction

**Args:** `--input_data_path=activity_data.csv --input_sep=, --output_model_path=svc_model.pkl --output_json_path=svc_metrics.json --features_col=mol_desc_1:mol_desc_20 --target_col=activity --kernel=rbf --C=1.0 --gamma=scale --test_size=0.25 --random_state=456`

**Explanation:** Trains an SVM with RBF kernel to predict ligand activity (binary active/inactive) based on molecular descriptors, outputting model performance metrics in JSON format.

### Perform hierarchical clustering on sequence similarity data

**Args:** `--input_data_path=similarity_matrix.csv --input_sep=, --output_json_path=hierarchical_results.json --output_csv_path=cluster_assignments.csv --features_col=col_1:col_N --linkage=ward --n_clusters=3`

**Explanation:** Hierarchical clustering using Ward linkage produces 3 clusters from the pairwise sequence similarity data. The cluster assignments are written to CSV while dendrogram metrics go to JSON.