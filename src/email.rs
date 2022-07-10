use crate::domain::SanitizedEmail;
use reqwest::Client;
use secrecy::ExposeSecret;
use secrecy::Secret;

#[derive(Debug)]
pub struct SendError;

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SanitizedEmail,
    auth_token: Secret<String>,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SanitizedEmail, auth_token: Secret<String>, timeout: std::time::Duration) -> Self {
        Self {
            http_client: Client::builder().timeout(timeout).build().expect("Failed to build client."),
            base_url,
            sender,
            auth_token,
        }
    }

    pub async fn send(
        &self,
        recipient: SanitizedEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);

        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject: subject,
            html_body: html_content,
            text_body: text_content,
        };

        self.http_client
            .post(&url)
            .header("X-Postmark-Server-Token", self.auth_token.expose_secret())
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}
