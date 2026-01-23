use crate::data::{
    Literal, ast::Interval, error_info::ErrorInfo, position::Position, primitive::Data,
    primitive::PrimitiveType,
};
use crate::error_format::gen_error_info;
use lettre::{
    message::{Mailbox, MultiPart, SinglePart, header},
    transport::smtp::authentication::{Credentials, Mechanism},
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::BuildHasher;
////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn format_email_value<'a, S: BuildHasher>(
    email: &'a HashMap<String, Literal, S>,
    value: &'a str,
    additional_info: &'a str,
    data: &'a Data,
    interval: Interval,
) -> Result<Option<&'a String>, ErrorInfo> {
    let error_message =
        format!("email [{value}] value need to be of type String {additional_info}");

    email
        .get(value)
        .map(|lit| {
            if lit.primitive.get_type() != PrimitiveType::PrimitiveString {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    error_message,
                ));
            }

            Literal::get_value::<String, _>(
                &lit.primitive,
                &data.context.flow,
                lit.interval,
                error_message,
            )
        })
        .transpose()
}

fn parse_email(email_str: &str, data: &Data, interval: Interval) -> Result<Mailbox, ErrorInfo> {
    match email_str.parse::<Mailbox>() {
        Ok(mbox) => Ok(mbox),
        Err(e) => Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("Invalid email format: {e:?}"),
        )),
    }
}

fn get_value<'a, T, E: Into<Cow<'static, str>>>(
    value: Option<&'a Literal>,
    data: &Data,
    error_message: E,
    interval: Interval,
) -> Result<&'a T, ErrorInfo>
where
    T: 'static,
{
    match value {
        Some(lit) => Literal::get_value::<T, _>(
            &lit.primitive,
            &data.context.flow,
            lit.interval,
            error_message,
        ),
        None => Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            error_message.into().into_owned(),
        )),
    }
}

// by default letter will use Mechanism::Plain and Mechanism::Login

// PLAIN authentication mechanism, defined in
// [RFC 4616](https://tools.ietf.org/html/rfc4616)
//   Plain,

// LOGIN authentication mechanism
// Obsolete but needed for some providers (like office365)
// Defined in [draft-murchison-sasl-login-00](https://www.ietf.org/archive/id/draft-murchison-sasl-login-00.txt).
//  Login, // AUTH LOGIN => LOGIN

// Non-standard XOAUTH2 mechanism, defined in
// [xoauth2-protocol](https://developers.google.com/gmail/imap/xoauth2-protocol)
// Xoauth2,
fn get_auth_mechanisms<S: BuildHasher>(
    object: &HashMap<String, Literal, S>,
    data: &Data,
    interval: Interval,
) -> Option<Vec<Mechanism>> {
    let auth_values = get_value::<HashMap<String, Literal>, _>(
        object.get("auth_mechanisms"),
        data,
        String::new(),
        interval,
    )
    .ok()?;

    let mut vec = vec![];

    if auth_values.contains_key("PLAIN") {
        vec.push(Mechanism::Plain);
    }

    if auth_values.contains_key("AUTH LOGIN") {
        vec.push(Mechanism::Login);
    }

    if auth_values.contains_key("XOAUTH2") {
        vec.push(Mechanism::Xoauth2);
    }

    if vec.is_empty() { None } else { Some(vec) }
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

pub fn format_email<S: BuildHasher>(
    email: &HashMap<String, Literal, S>,
    data: &Data,
    interval: Interval,
) -> Result<lettre::Message, ErrorInfo> {
    let mut message_builder = lettre::Message::builder();

    let from = format_email_value(email, "from", "and a valid email", data, interval)?;
    if let Some(form) = from {
        let mbox = parse_email(form.as_ref(), data, interval)?;
        message_builder = message_builder.from(mbox);
    }

    let to = format_email_value(email, "to", "and a valid email", data, interval)?;
    if let Some(to) = to {
        let mbox = parse_email(to.as_ref(), data, interval)?;
        message_builder = message_builder.to(mbox);
    }

    let reply_to = format_email_value(email, "reply_to", "and a valid email", data, interval)?;
    if let Some(reply_to) = reply_to {
        let mbox = parse_email(reply_to.as_ref(), data, interval)?;
        message_builder = message_builder.reply_to(mbox);
    }

    let bcc = format_email_value(email, "bcc", "and a valid email", data, interval)?;
    if let Some(bcc) = bcc {
        let mbox = parse_email(bcc.as_ref(), data, interval)?;
        message_builder = message_builder.bcc(mbox);
    }

    let cc = format_email_value(email, "cc", "and a valid email", data, interval)?;
    if let Some(cc) = cc {
        let mbox = parse_email(cc.as_ref(), data, interval)?;
        message_builder = message_builder.cc(mbox);
    }

    let subject = format_email_value(email, "subject", "", data, interval)?;
    if let Some(subject) = subject {
        message_builder = message_builder.subject(subject.clone());
    }

    let text = format_email_value(email, "text", "", data, interval)?;
    let html = format_email_value(email, "html", "", data, interval)?;

    if text.is_none() && html.is_none() {
        return Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            "email text/html parameter is mandatory".to_owned(),
        ));
    }

    let mut multipart = MultiPart::alternative().build();

    if let Some(text) = text {
        multipart = multipart.singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_PLAIN)
                .body(String::from(text)),
        );
    }
    if let Some(html) = html {
        multipart = multipart.singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_HTML)
                .body(String::from(html)),
        );
    }

    match message_builder.multipart(multipart) {
        Ok(message) => Ok(message),
        Err(_) => Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            "missing mandatory email parameter [from] or [to]".to_owned(),
        )),
    }
}

pub fn get_mailer<S: BuildHasher>(
    object: &mut HashMap<String, Literal, S>,
    data: &Data,
    interval: Interval,
) -> Result<lettre::SmtpTransport, ErrorInfo> {
    let username = get_value::<String, _>(
        object.get("username"),
        data,
        "username is missing or invalid type",
        interval,
    )?;
    let password = get_value::<String, _>(
        object.get("password"),
        data,
        "password is missing or invalid type",
        interval,
    )?;

    let auth_mechanisms = get_auth_mechanisms(object, data, interval);

    let starttls = if let Some(starttls) = object.get("starttls") {
        Literal::cast_value::<bool>(starttls.primitive.as_ref())
            .copied()
            .ok_or_else(|| {
                gen_error_info(
                    Position::new(interval, &data.context.flow),
                    "invalid starttls value".to_owned(),
                )
            })?
    } else {
        false
    };

    // Set the default port to [465] for TLS connections.
    // [RFC8314](https://tools.ietf.org/html/rfc8314)
    let port = if let Some(port) = object.get("port") {
        Literal::cast_value::<i64>(port.primitive.as_ref())
            .and_then(num_traits::ToPrimitive::to_u16)
            .ok_or_else(|| {
                gen_error_info(
                    Position::new(interval, &data.context.flow),
                    "invalid port".to_owned(),
                )
            })?
    } else {
        465
    };
    let smtp_server = get_value::<String, _>(
        object.get("smtp_server"),
        data,
        "SMTP server address is missing or invalid type",
        interval,
    )?;

    let credentials = Credentials::new(username.clone(), password.clone());

    let is_tls = if let Some(is_tls) = object.get("tls") {
        Literal::cast_value::<bool>(is_tls.primitive.as_ref())
            .copied()
            .ok_or_else(|| {
                gen_error_info(
                    Position::new(interval, &data.context.flow),
                    "invalid tls value".to_owned(),
                )
            })?
    } else {
        true
    };

    if is_tls {
        let smtp_builder = if starttls {
            lettre::SmtpTransport::starttls_relay(smtp_server)
        } else {
            lettre::SmtpTransport::relay(smtp_server)
        };

        let Ok(smtp_server) = smtp_builder else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                "invalid SMTP address".to_owned(),
            ));
        };
        let mut smtp_builder = smtp_server.credentials(credentials).port(port);

        if let Some(auth_mechanisms) = auth_mechanisms {
            smtp_builder = smtp_builder.authentication(auth_mechanisms);
        }

        Ok(smtp_builder.build())
    } else {
        let mailer = lettre::SmtpTransport::builder_dangerous(smtp_server)
            .credentials(credentials)
            .port(port)
            .build();

        Ok(mailer)
    }
}

pub fn get_auth_mechanism(
    lit: &Literal,
    data: &Data,
    interval: Interval,
    usage: &str,
) -> Result<String, ErrorInfo> {
    let value = Literal::get_value::<String, _>(
        &lit.primitive,
        &data.context.flow,
        lit.interval,
        format!("usage: {usage}"),
    )?;

    // "XOAUTH2", "AUTH LOGIN", "PLAIN"
    match value.to_lowercase() {
        value if value == "PLAIN".to_lowercase() => Ok("PLAIN".to_owned()),
        value if value == "AUTH LOGIN".to_lowercase() => Ok("AUTH LOGIN".to_owned()),
        value if value == "XOAUTH2".to_lowercase() => Ok("XOAUTH2".to_owned()),
        _ => Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            usage.to_owned(),
        )),
    }
}
