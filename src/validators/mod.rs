mod validate_extra;

pub use validate_extra::ValidateExtra;

pub struct Validators;

#[cfg_attr(test, mockall::automock)]
impl Validators {
    pub fn new() -> Self {
        Self
    }
}
