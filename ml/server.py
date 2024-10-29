from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import tensorflow as tf
import numpy as np

model_path = "test.keras"
model = tf.keras.models.load_model(model_path)

app = FastAPI()


class PredictionRequest(BaseModel):
    input_data: list


class PredictionResponse(BaseModel):
    predictions: list


@app.post("/predict", response_model=PredictionResponse)
async def predict(request: PredictionRequest):
    try:
        # Preprocess input data
        input_array = np.array(request.input_data)
        # Ensure input shape matches model requirements
        if len(input_array.shape) == 1:
            input_array = np.expand_dims(input_array, axis=0)

        # Make predictions
        predictions = model.predict(input_array)

        # Format predictions as list to match response model
        return PredictionResponse(predictions=predictions.tolist())

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
