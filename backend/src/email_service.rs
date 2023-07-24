use flume::{Receiver, Sender};
use handlebars::Handlebars;
use lettre::message::{Body, MessageBuilder};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::Error as SmtpError;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message};
use rust_embed::RustEmbed;
use serde::Serialize;
use std::io;
use tracing::{debug, info, warn};
use utils::config::EmailSetting;
use utils::database::EmailAddress;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/resources/emails"]
pub struct EmailTemplates;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailDebug {
    pub to: String,
    pub subject: &'static str,
}

#[derive(Debug, Clone)]
pub struct EmailRequest {
    pub debug_info: EmailDebug,
    pub message: Message,
}
impl EmailRequest {
    pub fn new(debug_info: EmailDebug, message: Message) -> Self {
        Self {
            debug_info,
            message,
        }
    }
}
pub trait Email: Serialize {
    fn template() -> &'static str;

    fn subject() -> &'static str;

    fn debug_info(self) -> EmailDebug;
}
#[derive(Debug)]
pub struct EmailAccess {
    queue: Sender<EmailRequest>,
    message_builder: MessageBuilder,
    email_handlebars: Handlebars<'static>,
}
impl EmailAccess {
    #[inline]
    pub fn send(&self, debug_info: EmailDebug, message: Message) {
        let request = EmailRequest::new(debug_info, message);
        if let Err(error) = self.queue.send(request) {
            warn!("Email Queue Error: {}", error);
        };
    }
    pub fn get_handlebars(&self) -> &Handlebars<'static> {
        &self.email_handlebars
    }
    #[inline]
    pub fn build_body<E: Email>(&self, data: &E) -> Body {
        self.email_handlebars
            .render(E::template(), &data)
            .map(|e| Body::new(e))
            .expect("Unable to render email body. This is a bug. Please report it.")
    }
    #[inline]
    pub fn prep_builder(&self) -> MessageBuilder {
        self.message_builder.clone()
    }
    pub fn send_one_fn(&self, to: EmailAddress, data: impl Email) {
        let body = self.build_body(&data);

        let message = self
            .prep_builder()
            .to(to.into())
            .body(body)
            .expect("Unable to build email");
        self.send(data.debug_info(), message);
    }
}

type Transport = AsyncSmtpTransport<lettre::Tokio1Executor>;
#[derive(Debug)]
pub struct EmailService {
    queue: Receiver<EmailRequest>,
    transport: Transport,
}

impl EmailService {
    pub async fn start(email: EmailSetting) -> io::Result<Option<EmailAccess>> {
        let Some(transport) = Self::build_connection(email.clone()).await.map_err(|e|{
            io::Error::new(io::ErrorKind::Other, format!("Email Transport Error: {}", e))
        })? else{
            return Ok(None);
        };

        let mut message_builder = Message::builder().from(email.from.parse().unwrap());
        if let Some(reply_to) = &email.reply_to {
            message_builder = message_builder.reply_to(reply_to.parse().unwrap());
        }

        let mut email_handlebars = Handlebars::new();
        email_handlebars
            .register_embed_templates::<EmailTemplates>()
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Email Handlebars Error: {:?}", e),
                )
            })?;

        let (sender, receiver) = flume::bounded(100);
        let email_service = EmailService {
            queue: receiver,
            transport,
        };
        actix_rt::spawn(async move {
            email_service.run().await;
        });
        Ok(Some(EmailAccess {
            queue: sender,
            message_builder,
            email_handlebars,
        }))
    }

    async fn run(self) {
        let mut connection = self.transport;
        let queue = self.queue;
        while let Ok(value) = queue.recv_async().await {
            Self::send_email(&mut connection, value).await;
        }
    }

    async fn send_email(connection: &mut Transport, value: EmailRequest) {
        let EmailRequest {
            debug_info,
            message,
        } = value;

        debug!("Sending Email: {:?}", debug_info);
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
