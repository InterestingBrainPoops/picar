import onnxruntime as ort
import numpy as np
ort_session = ort.InferenceSession("../driver.onnx")

outputs = ort_session.run(
    None,
    {"onnx::MatMul_0": np.array([0.5, 0.3]).astype(np.float32)},
)
print(outputs)