use crate::services::service_error::ServiceError;

mod services;

fn main() {
    println!("{:?}", ServiceError::DomainValidationError("Soy un error de validacion de dominio".to_string()));
}
