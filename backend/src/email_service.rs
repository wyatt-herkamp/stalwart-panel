use flume::{Receiver, Sender};
use lettre::message::MessageBuilder;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::Error as SmtpError;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message};

use tracing::{debug, info, warn};
use utils::config::EmailSetting;

/// For logging purposes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailType {
    ToUser {
        username: String,
        subject: &'static str,
    },
}
impl EmailType {
    pub fn reset_password(username: String) -> Self {
        Self::ToUser {
            username,
            subject: "Reset Password",
        }
    }
}
#[derive(Debug, Clone)]
pub struct EmailRequest {
    pub email_type: EmailType,
    pub message: Message,
}
impl EmailRequest {
    pub fn new(email_type: EmailType, message: Message) -> Self {
        Self {
            email_type,
            message,
        }
    }
}
#[derive(Debug)]
pub struct EmailAccess {
    queue: Sender<EmailRequest>,
    message_builder: MessageBuilder,
}
impl EmailAccess {
    pub fn new(queue: Sender<EmailRequest>, message_builder: MessageBuilder) -> Self {
        Self {
            queue,
            message_builder,
        }
    }
    pub fn send(&self, email_type: EmailType, message: Message) {
        let request = EmailRequest::new(email_type, message);
        if let Err(error) = self.queue.send(request) {
            warn!("Email Queue Error: {}", error);
        };
    }
    pub fn prep_builder(&self) -> MessageBuilder {
        self.message_builder.clone()
    }
}

type Transport = AsyncSmtpTransport<lettre::Tokio1Executor>;
#[derive(Debug)]
pub struct EmailService {
    queue: Receiver<EmailRequest>,
    transport: Transport,
}

impl EmailService {
    pub async fn start(email: EmailSetting) -> Result<Option<EmailAccess>, SmtpError> {
        let Some(transport) = Self::build_connection(email.clone()).await? else{
            return Ok(None);
        };

        let mut message_builder = Message::builder().from(email.from.parse().unwrap());
        if let Some(reply_to) = &email.reply_to {
            message_builder = message_builder.reply_to(reply_to.parse().unwrap());
        }

        let (sender, receiver) = flume::bounded(100);
        let email_service = EmailService {
            queue: receiver,
            transport,
        };
        actix_rt::spawn(async move {
            email_service.run().await;
        });
        Ok(Some(EmailAccess::new(sender, message_builder)))
    }

    async fn run(mut self) {
        let mut connection = self.transport;
        let mut queue = self.queue;
        while let Ok(value) = queue.recv_async().await {
            Self::send_email(&mut connection, value).await;
        }
    }

    async fn send_email(connection: &mut Transport, value: EmailRequest) {
        let EmailRequest {
            email_type,
            message,
        } = value;

        debug!("Sending Email: {:?}", email_type);
        match connection.send(message).await {
            Ok(ok) => {
                if ok.is_positive() {
                    info!("Email Sent Successfully");
                } else {
                    warn!("Email Send Error: {:?}", ok);
                }
            }
            Err(err) => {
                warn!("Email Send Error: {}", err);
            }
        }
    }
    async fn build_connection(email: EmailSetting) -> Result<Option<Transport>, SmtpError> {
        let transport = Transport::starttls_relay(email.host.as_str())?
            .credentials(Credentials::new(
                email.username.clone(),
                email.password.clone(),
            ))
            .build();
        if transport.test_connection().await? {
            warn!("Email Transport Test Connection Failed");
            warn!("Please ensure that stalwart has already been configured");
            return Ok(None);
        }
        Ok(Some(transport))
    }
}
