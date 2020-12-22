use alto::{Alto, Context, OutputDevice};

pub struct OpenalData {
    alto: Alto,
    device: OutputDevice,
    ctx: Context,
}

impl Default for OpenalData {
    fn default() -> Self {
        let alto = Alto::load_default().expect("failed to get alto.");
        let device = alto.open(None).expect("open device failed.");
        let ctx = device.new_context(None).expect("get context failed.");
        Self {
            alto,
            device,
            ctx,
        }
    }
}


impl OpenalData {
    pub fn play(&self, name: &String) {}
}