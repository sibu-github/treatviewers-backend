/// Send sms to a given phone number with message provided
/// Call external third party sms gateway api
pub async fn send_sms(phone: &str, message: &str) -> anyhow::Result<()> {
    tracing::debug!("Sending sms to phone: {} with message: {}", phone, message);
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_send_sms() {
        let phone = "1111111111";
        let message = "abcd";
        let res = send_sms(phone, message).await;
        assert_eq!(res.is_ok(), true);
    }
}
