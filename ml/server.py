import logging
import time
from pathlib import Path
from typing import Optional

import numpy as np
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

try:
    import keras
except ImportError:  # pragma: no cover - optional dependency
    keras = None  # type: ignore

MODEL_PATH = Path("models/rent_model.keras")
MODEL = None

LOGGER = logging.getLogger("apato.ml-server")
if not LOGGER.handlers:
    handler = logging.StreamHandler()
    handler.setFormatter(
        logging.Formatter("%(asctime)s %(levelname)s %(name)s %(message)s")
    )
    LOGGER.addHandler(handler)
LOGGER.setLevel(logging.INFO)
LOGGER.propagate = False

if keras is not None and MODEL_PATH.exists():
    MODEL = keras.models.load_model(MODEL_PATH)
    LOGGER.info("Loaded ML model from %s", MODEL_PATH)
else:
    LOGGER.warning(
        "No ML model found at %s; falling back to heuristic estimates", MODEL_PATH
    )


app = FastAPI(title="Apato ML Service", version="1.0.0")


class RentPredictionRequest(BaseModel):
    location_id: int
    location_level: int
    size: float
    rooms: Optional[int] = None
    price: Optional[float] = None
    maintenance_fee: Optional[float] = None


class RentPredictionResponse(BaseModel):
    rent: int


def _baseline_rent_estimate(size: float, rooms: Optional[int], maintenance_fee: Optional[float]) -> int:
    """Fallback heuristic that approximates the historical Rust calculation."""
    normalized_size = max(size, 1.0)
    base_rate = 15.0  # €/m2 baseline
    room_multiplier = 1.0 + (0.05 * (rooms or 0))
    maintenance_adjustment = -0.5 * (maintenance_fee or 0.0) / normalized_size
    estimate = normalized_size * base_rate * room_multiplier + maintenance_adjustment
    return max(int(round(estimate)), 0)


@app.post("/predict", response_model=RentPredictionResponse)
async def predict(request: RentPredictionRequest):
    start_time = time.perf_counter()
    LOGGER.info(
        "Received prediction request: location_id=%s location_level=%s size=%.2f rooms=%s price=%s maintenance_fee=%s",
        request.location_id,
        request.location_level,
        request.size,
        request.rooms,
        request.price,
        request.maintenance_fee,
    )
    try:
        if MODEL is None:
            rent_estimate = _baseline_rent_estimate(request.size, request.rooms, request.maintenance_fee)
            duration = time.perf_counter() - start_time
            LOGGER.info(
                "Responding with heuristic estimate rent=%s duration=%.4fs",
                rent_estimate,
                duration,
            )
            return RentPredictionResponse(rent=rent_estimate)

        input_vector = np.array(
            [
                [
                    request.location_id,
                    request.location_level,
                    request.size,
                    request.rooms or 0,
                    request.price or 0.0,
                    request.maintenance_fee or 0.0,
                ]
            ]
        )
        predictions = MODEL.predict(input_vector)
        rent_value = int(round(float(predictions.squeeze())))
        rent_value = max(rent_value, 0)
        duration = time.perf_counter() - start_time
        LOGGER.info(
            "Responding with ML prediction rent=%s duration=%.4fs",
            rent_value,
            duration,
        )
        return RentPredictionResponse(rent=rent_value)
    except Exception as exc:  # pragma: no cover - defensive
        duration = time.perf_counter() - start_time
        LOGGER.exception("Prediction failed after %.4fs", duration)
        raise HTTPException(status_code=500, detail=str(exc)) from exc
