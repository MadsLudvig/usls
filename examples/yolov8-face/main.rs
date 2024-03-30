use usls::{models::YOLO, DataLoader, Options};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build model
    let options = Options::default()
        .with_model("../models/yolov8n-face-dyn-f16.onnx")
        .with_i00((1, 1, 4).into())
        .with_i02((416, 640, 800).into())
        .with_i03((416, 640, 800).into())
        .with_confs(&[0.15])
        .with_saveout("YOLOv8-Face")
        .with_profile(false);
    let mut model = YOLO::new(&options)?;

    // load image
    let x = DataLoader::try_read("./assets/kids.jpg")?;

    // run
    let _y = model.run(&[x])?;

    Ok(())
}
