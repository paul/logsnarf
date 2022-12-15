use crate::metric::Metric;

pub trait Adapter {
    fn write(&self, metrics: Vec<Metric>);
}
