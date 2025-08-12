import argparse
import json
import signal
import sys
from datetime import datetime
import cv2
from ultralytics import YOLO

# Constants
MODEL_NAME = "yolov8n.pt"
CONFIDENCE_THRESHOLD = 0.5
FRAME_SKIP = 2
CAMERA_INDEX = 0
TARGET_LABELS = ("cell phone", "phone", "mobile", "camera", "camcorder")

# Graceful exit flag
running = True


def signal_handler(sig, frame):
    global running
    running = False
    print("\nStopping detection...", file=sys.stderr)


signal.signal(signal.SIGINT, signal_handler)
signal.signal(signal.SIGTERM, signal_handler)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--device", type=int, default=CAMERA_INDEX, help="Camera index")
    parser.add_argument("--conf-threshold", type=float, default=CONFIDENCE_THRESHOLD, help="Confidence threshold")
    parser.add_argument("--frame-skip", type=int, default=FRAME_SKIP, help="Number of frames to skip between detections")
    args = parser.parse_args()

    model = YOLO(MODEL_NAME)
    cap = cv2.VideoCapture(args.device)

    if not cap.isOpened():
        print("Error: Cannot open camera", file=sys.stderr)
        sys.exit(1)

    frame_count = 0

    while running:
        ret, frame = cap.read()
        if not ret:
            break

        frame_count += 1
        if frame_count % args.frame_skip != 0:
            continue

        results = model(frame, verbose=False)
        for r in results:
            boxes = r.boxes
            for box in boxes:
                label_index = int(box.cls)
                label = r.names[label_index].lower()
                confidence = float(box.conf)

                if label in TARGET_LABELS and confidence >= args.conf_threshold:
                    # JSON output for logging
                    event = {
                        "label": label,
                        "confidence": confidence,
                        "timestamp": datetime.now().isoformat()
                    }
                    print(json.dumps(event), flush=True)

                    # Plain text output for external readers
                    if label in ("cell phone", "phone", "mobile"):
                        print("Phone detected", flush=True)
                    elif label in ("camera", "camcorder"):
                        print("Camera detected", flush=True)

    cap.release()


if __name__ == "__main__":
    main()
