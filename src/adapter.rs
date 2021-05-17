use crate::decoder::Metric;

pub trait Adapter {
    fn write(&self, metrics: Vec<Metric>);
}
