use alto::{Alto, Context, OutputDevice};

pub struct OpenalData {
    alto: Alto,
    device: OutputDevice,
    ctx: Context,
}

impl OpenalData {
    pub(crate) fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let alto = Alto::load_default()?;
        let device = alto.open(None)?;
        let ctx = device.new_context(None)?;
        Ok(Self {
            alto,
            device,
            ctx,
        })
    }
}


impl OpenalData {
    pub fn play(&self, name: &String) {}
}