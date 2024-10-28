import pandas as pd
import numpy as np
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import OneHotEncoder, StandardScaler
from sklearn.compose import ColumnTransformer
from sklearn.pipeline import Pipeline
from tensorflow.keras.models import Sequential
from tensorflow.keras.layers import Dense, Dropout
from tensorflow.keras.callbacks import EarlyStopping


def build_model(input_dim):
    """
    Builds a Sequential neural network model for regression.

    Args:
        input_dim (int): Number of input features.

    Returns:
        model (Sequential): Compiled Keras model.
    """
    model = Sequential()
    model.add(Dense(128, input_dim=input_dim, activation="relu"))
    model.add(Dropout(0.2))
    model.add(Dense(64, activation="relu"))
    model.add(Dropout(0.2))
    model.add(Dense(32, activation="relu"))
    model.add(Dense(1, activation="linear"))  # Linear activation for regression
    return model


def main():
    # load data
    df = pd.read_csv("rental_data_finland.csv")
    df = df.dropna(subset=["rent_amount"])

    # Features
    numerical_features = [
        "size_m2",
        "num_rooms",
        "build_year",
        "floor",
        "latitude",
        "longitude",
    ]
    categorical_features = ["zip_code", "city", "district"]

    # Fill missing numerical features with median
    for col in numerical_features:
        if df[col].isnull().sum() > 0:
            median = df[col].median()
            df[col].fillna(median, inplace=True)

    # Fill missing categorical features with mode
    for col in categorical_features:
        if df[col].isnull().sum() > 0:
            mode = df[col].mode()[0]
            df[col].fillna(mode, inplace=True)

    # FEATURES AND LABELS
    X = df.drop(["rent_amount"], axis=1)
    y = df["rent_amount"]

    # Define preprocessor
    preprocessor = ColumnTransformer(
        transformers=[
            ("num", StandardScaler(), numerical_features),
            ("cat", OneHotEncoder(handle_unknown="ignore"), categorical_features),
        ]
    )

    # Split the data
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.1, random_state=42
    )

    print(f"\nTraining Set: {X_train.shape}")
    print(f"Testing Set: {X_test.shape}")

    # Preprocess the data
    X_train_processed = preprocessor.fit_transform(X_train)
    X_test_processed = preprocessor.transform(X_test)

    # Get the number of input features after preprocessing
    input_dim = X_train_processed.shape[1]

    # Build and compile the model
    model = build_model(input_dim)
    model.compile(optimizer="adam", loss="mse", metrics=["mae"])

    # Define Early Stopping
    early_stop = EarlyStopping(
        monitor="val_loss", patience=10, restore_best_weights=True
    )

    # Train the model
    model.fit(
        X_train_processed,
        y_train,
        epochs=100,
        batch_size=32,
        validation_split=0.2,
        callbacks=[early_stop],
        verbose=1,
    )

    model.save("rental_prediction.keras")
    print("Model saved.")


if __name__ == "__main__":
    main()
