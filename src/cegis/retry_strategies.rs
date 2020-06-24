use super::CEGISState;
use log::warn;

pub trait RetryStrategy {
    fn succeed(&mut self, state: &CEGISState);
    fn fail_and_retry(&mut self, state: &mut CEGISState) -> bool;
}

pub struct SimpleRetryStrategy {
    failed_times: usize,
    retry_amount: usize
}

impl SimpleRetryStrategy {
    pub fn new(retry_amount: usize) -> Self {
        SimpleRetryStrategy {
            failed_times: 0,
            retry_amount: retry_amount
        }
    }
}

impl RetryStrategy for SimpleRetryStrategy {
    fn succeed(&mut self, _state: &CEGISState) {
        self.failed_times = 0;
    }
    fn fail_and_retry(&mut self, _state: &mut CEGISState) -> bool {
        self.failed_times += 1;
        warn!(target: "SimpleRetryStrategy", "Total failed times: {}, retry amount: {}", self.failed_times, self.retry_amount);
        self.failed_times > self.retry_amount
    }
}

pub struct NeverRetryStrategy;

impl RetryStrategy for NeverRetryStrategy {
    fn succeed(&mut self, _state: &CEGISState) {}
    fn fail_and_retry(&mut self, _state: &mut CEGISState) -> bool {
        warn!(target: "NeverRetryStrategy", "Never retrying in this strategy");
        false
    }
}