use accel_stepper::Device;
use esp_idf_hal::gpio::{OutputPin, PinDriver, Output};

pub struct Stepper<T: OutputPin, U: OutputPin, V: OutputPin, W: OutputPin> {
    p1: PinDriver<'static, T, Output>,
    p2: PinDriver<'static, U, Output>,
    p3: PinDriver<'static, V, Output>,
    p4: PinDriver<'static, W, Output>,
}

impl<T: OutputPin, U: OutputPin, V: OutputPin, W: OutputPin> Device for Stepper<T,U,V,W> {
    type Error = ();

    fn step(&mut self, ctx: &accel_stepper::StepContext) -> Result<(), Self::Error> {
        self.step(ctx.position);
        Ok(())
    }
}

impl <T: OutputPin, U: OutputPin, V: OutputPin, W: OutputPin> Stepper<T,U,V,W> {
    pub fn new(p1: T, p2: U, p3: V, p4: W)->Self {
        Self { p1: PinDriver::output(p1).unwrap(),
             p2: PinDriver::output(p2).unwrap(),
             p3: PinDriver::output(p3).unwrap(),
             p4: PinDriver::output(p4).unwrap() 
        }
    }

    pub fn stop(&mut self) {
        self.p1.set_low().unwrap();
        self.p2.set_low().unwrap();
        self.p3.set_low().unwrap();
        self.p4.set_low().unwrap();
    }

    pub fn step(&mut self, step: i64) {
        if step.rem_euclid(4) == 0 {
            self.p1.set_high().unwrap();
            self.p2.set_low().unwrap();
            self.p3.set_low().unwrap();
            self.p4.set_low().unwrap();
        }
        if step.rem_euclid(4) == 1 {
            self.p1.set_low().unwrap();
            self.p2.set_high().unwrap();
            self.p3.set_low().unwrap();
            self.p4.set_low().unwrap();
        }
        if step.rem_euclid(4) == 2 {
            self.p1.set_low().unwrap();
            self.p2.set_low().unwrap();
            self.p3.set_high().unwrap();
            self.p4.set_low().unwrap();
        }
        if step.rem_euclid(4) == 3 {
            self.p1.set_low().unwrap();
            self.p2.set_low().unwrap();
            self.p3.set_low().unwrap();
            self.p4.set_high().unwrap();
        }

    }
}