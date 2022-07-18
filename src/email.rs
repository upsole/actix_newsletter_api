use crate::domain::{ParsedAccount, SanitizedEmail};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::Error;
use lettre::{Message, SmtpTransport, Transport};
use uuid::Uuid;

#[derive(Debug)]
pub struct SendError;

#[derive(Debug)]
pub struct LettreClientError;

#[derive(Clone)]
pub struct EmailClient {
    lettre_transport: SmtpTransport,
    sender: SanitizedEmail,
    username: String,
    base_url: String,
}


impl EmailClient {
    pub fn new(
        server: String,
        sender: SanitizedEmail,
        user: String,
        password: String,
        base_url: String,
    ) -> Result<EmailClient, LettreClientError> {
        let sasl_credentials = Credentials::new(user.clone(), password);
        let mailer = SmtpTransport::relay(&server)
            .expect("Could not connect to SMTP server")
            .credentials(sasl_credentials)
            .build();
        match mailer.test_connection() {
            Ok(_) => Ok(Self {
                lettre_transport: mailer,
                sender,
                username: user,
                base_url,
            }),
            Err(_) => Err(LettreClientError),
        }
    }
    pub fn send_confirmation(
        &self,
        recipient: ParsedAccount,
        auth_token: Uuid,
    ) -> Result<(), Error> {
        let email = Message::builder()
            .from(
                format!("{} <{}>", self.username, self.sender)
                    .parse()
                    .expect("Could not parse sender user - <email>"),
            )
            .to(format!("{} <{}>", recipient.name, recipient.email)
                .parse()
                .expect("Could not parse recipient user - <email>"))
            .subject("Confirm your subscription to upsol.me")
            .body(format!(
                "Hello {},\nClick here to confirm your account\n\n {}/confirm/{}",
                recipient.name, self.base_url, auth_token
            ))
            .expect("Error building the email");
        match self.lettre_transport.send(&email) {
            Ok(_) => Ok(tracing::info!("Confirmation mail sent")),
            Err(e) => Err(e),
        }
    }
}
