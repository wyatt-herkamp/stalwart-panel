use flume::{Receiver, Sender};
use handlebars::Handlebars;

use futures_util::FutureExt;
use futures_util::StreamExt;
use lettre::message::{header, MessageBuilder, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::Error as SmtpError;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message};
use rust_embed::RustEmbed;
use serde::Serialize;
use std::fmt::{Debug, Formatter};
use std::io;

use tracing::log::log_enabled;
use tracing::{debug, error, info, warn};
use utils::config::{EmailEncryption, EmailSetting};
use utils::database::EmailAddress;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/resources/emails"]
pub struct EmailTemplates;
#[derive(Clone, PartialEq, Eq)]
pub struct EmailDebug {
    pub to: String,
    pub subject: &'static str,
}
impl Debug for EmailDebug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ to: {}, subject: {} }}", self.to, self.subject)
    }
}

#[derive(Debug, Clone)]
pub struct EmailRequest {
    pub debug_info: Option<EmailDebug>,
    pub message: Message,
}
impl EmailRequest {
    pub fn new(debug_info: Option<EmailDebug>, message: Message) -> Self {
        Self {
            debug_info,
            message,
        }
    }
}
macro_rules! template {
    ($template:expr) => {
        fn template_html() -> &'static str {
            concat!($template, ".html")
        }
        fn template_txt() -> &'static str {
            concat!($template, ".txt")
        }
    };
}
pub(crate) use template;
pub trait Email: Serialize {
    /// template().html and template().txt must exist in the resources/emails folder
    fn template_html() -> &'static str;

    fn template_txt() -> &'static str;

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
    /// Adds a new Email to the queue to be sent
    ///
    /// # Arguments
    /// debug_info - If Debug Logging is is enabled this should be Some(EmailDebug). Otherwise it should be None
    /// message - The message to be sent
    #[inline]
    pub fn send(&self, debug_info: Option<EmailDebug>, message: Message) {
        let request = EmailRequest::new(debug_info, message);
        if let Err(error) = self.queue.send(request) {
            warn!("Email Queue Error: {}", error);
        };
    }
    pub fn get_handlebars(&self) -> &Handlebars<'static> {
        &self.email_handlebars
    }
    #[inline]
    pub fn build_body<E: Email>(&self, data: &E) -> MultiPart {
        let multipart = MultiPart::alternative();
        let mut multipart = match self.email_handlebars.render(E::template_txt(), &data) {
            Ok(ok) => multipart.singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_PLAIN)
                    .body(ok),
            ),
            Err(error) => {
                error!("Email Error: {}", error);
                multipart.build()
            }
        };
        match self.email_handlebars.render(E::template_html(), &data) {
            Ok(ok) => {
                multipart = multipart.singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(ok),
                );
            }
            Err(err) => {
                error!("Email Error: {}", err);
            }
        }

        multipart
    }
    #[inline]
    pub fn prep_builder(&self) -> MessageBuilder {
        self.message_builder.clone()
    }
    pub fn send_one_fn(&self, to: EmailAddress, data: impl Email) {
        let body = self.build_body(&data);

        let message = match self.prep_builder().to(to.into()).multipart(body) {
            Ok(ok) => ok,
            Err(value) => {
                error!("Email Error: {}", value);
                return;
            }
        };
        let debug = if log_enabled!(tracing::log::Level::Debug) {
            Some(data.debug_info())
        } else {
            None
        };
        self.send(debug, message);
    }
}

type Transport = AsyncSmtpTransport<lettre::Tokio1Executor>;
#[derive(Debug)]
pub struct EmailService;
impl EmailService {
    pub async fn start(email: EmailSetting) -> io::Result<Option<EmailAccess>> {
        let Some(transport) = Self::build_connection(email.clone()).await.map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Email Transport Error: {}", e),
            )
        })?
        else {
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
        actix_rt::spawn(async move {
            Self::run(transport, receiver).await;
        });
        Ok(Some(EmailAccess {
            queue: sender,
            message_builder,
            email_handlebars,
        }))
    }

    async fn run(connection: Transport, queue: Receiver<EmailRequest>) {
        let mut shutdown_hook = Box::pin(tokio::signal::ctrl_c().fuse());
        let mut queue_async = queue.stream().fuse();
        loop {
            futures_util::select! {
                _ = shutdown_hook => {
                    debug!("Shutdown Signal Received. Stopping Email Service");
                    break;
                }
                value = queue_async.next() => {
                    if let Some(value) = value {
                        Self::send_email(&connection, value).await;
                    }
                }
            }
        }
        drop(queue_async);
        // This could be a problem.
        // I don't know if once Sigkill has been sent Tokio runtime will still be running
        // Also the active Password Reset Tokens are stored in memory.
        // So sending these emails is unnecessary
        if !queue.is_empty() {
            info!("Email Queue is not empty. Sending remaining emails");
            while let Ok(value) = queue.try_recv() {
                Self::send_email(&connection, value).await;
            }
        }
        info!("Email Service has been stopped")
    }

    async fn send_email(connection: &Transport, value: EmailRequest) {
        let EmailRequest {
            debug_info,
            message,
        } = value;
        if let Some(debug_info) = &debug_info {
            debug!("Sending Email: {:?}", debug_info);
        }
        match connection.send(message).await {
            Ok(ok) => {
                if ok.is_positive() {
                    debug!("Email Sent Successfully");
                } else {
                    error!("Email Send Error for {:?}", debug_info);
                }
            }
            Err(err) => {
                error!("Email Send Error: {} for {:?}", err, debug_info);
            }
        }
    }
    async fn build_connection(email: EmailSetting) -> Result<Option<Transport>, SmtpError> {
        let credentials = Credentials::new(email.username.clone(), email.password.clone());
        let transport = match email.encryption {
            EmailEncryption::StartTLS => Transport::starttls_relay(email.host.as_str())?
                .credentials(credentials)
                .build(),
            _ => Transport::relay(email.host.as_str())?
                .credentials(credentials)
                .build(),
        };
        if !transport.test_connection().await? {
            warn!("Email Transport Test Connection Failed");
            warn!("Please ensure that stalwart has already been configured");
            return Ok(None);
        }
        Ok(Some(transport))
    }
}
