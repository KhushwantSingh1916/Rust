import sys
import json
import time
import signal
import argparse

import cv2
from ultralytics import YOLO

# --- config ---
MODEL = "yolov8n.pt"       # small, fast; ultralytics will download if missing
CONF_THRESHOLD = 0.6       # adjust to reduce false positives
FRAME_SKIP = 2             # process every Nth frame to reduce CPU
VIDEO_DEVICE = 0           # default webcam
# --------------

stop_requested = False

def on_exit(sig, frame):
    global stop_requested
    stop_requested = True

signal.signal(signal.SIGINT, on_exit)
signal.signal(signal.SIGTERM, on_exit)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--device", type=int, default=VIDEO_DEVICE)
    parser.add_argument("--conf", type=float, default=CONF_THRESHOLD)
    parser.add_argument("--skip", type=int, default=FRAME_SKIP)
    args = parser.parse_args()

    try:
        model = YOLO(MODEL)
    except Exception as e:
        print(json.dumps({"type":"error","message":f"model load failed: {e}"}), flush=True)
        return

    cap = cv2.VideoCapture(args.device, cv2.CAP_ANY)
    if not cap.isOpened():
        print(json.dumps({"type":"error","message":"camera_open_failed"}), flush=True)
        return

    frame_idx = 0
    try:
        while not stop_requested:
            ret, frame = cap.read()
            if not ret:
                time.sleep(0.05)
                continue
            frame_idx += 1
            if frame_idx % args.skip != 0:
                continue

            
            results = model(frame, imgsz=640, half=False, verbose=False)

            detected = False
            best_conf = 0.0
            best_label = None
            for r in results:
                boxes = getattr(r, "boxes", None)
                if boxes is None:
                    continue
                for box in boxes:
                    conf = float(box.conf[0])
                    cls = int(box.cls[0])
                    label = model.names[cls].lower() if model.names and cls in model.names else str(cls)
                   
                    if label in ("cell phone", "cellphone", "phone", "mobile", "camera", "camcorder"):
                        if conf >= args.conf:
                            detected = True
                            if conf > best_conf:
                                best_conf = conf
                                best_label = label

            if detected:
                event = {
                    "type": "detection",
                    "label": best_label or "device",
                    "confidence": float(best_conf),
                    "timestamp": time.time()
                }
                print(json.dumps(event), flush=True)

            time.sleep(0.01)

    finally:
        try:
            cap.release()
        except:
            pass

if __name__ == "__main__":
    main()
