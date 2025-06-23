use std::error::Error;
use std::fs;
use handlebars::Handlebars;
use lettre::transport::smtp::authentication::Credentials;

use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use lettre::message::header::ContentType;
use serde_derive::Serialize;
use crate::CONFIG;

pub struct EmailService{
    
}
impl EmailService {
    pub async  fn send_signup_email(reciever_email:String, code:String)->Result<(), Box<dyn Error>>{
        let mailer:AsyncSmtpTransport<Tokio1Executor> = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.zoho.com")
            .unwrap()
            .credentials(Credentials::new(CONFIG.email.clone(), CONFIG.email_password.clone()))
            .build();
        let template_path = "templates/signup_email_template.hbs";
        let template_content = fs::read_to_string(template_path)
            .expect("Failed to read email template file");
        #[derive(Serialize)]
        struct EmailContext {
            code: String,
        }
        let context = EmailContext {
            code: code.clone(),
        };

        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("email_template", &template_content)
            .expect("Failed to register template");
        let rendered_body = handlebars
            .render("email_template", &context)
            .expect("Failed to render template");

        let email = Message::builder()
            .from("hello@vhennus.com".parse().unwrap())
            .to(reciever_email.parse().unwrap())
            .subject("Signup Email")
            .header(ContentType::TEXT_HTML)
            .body(rendered_body)
            .unwrap();

        match mailer.send(email).await {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("Failed to send email: {}", e);
                Err(Box::new(e))
            }
        }
    }

    pub async  fn send_reset_password_email(reciever_email:String, code:String)->Result<(), Box<dyn Error>>{
        let mailer:AsyncSmtpTransport<Tokio1Executor> = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.zoho.com")
            .unwrap()
            .credentials(Credentials::new(CONFIG.email.clone(), CONFIG.email_password.clone()))
            .build();
        let template_path = "templates/reset_password_email.hbs";
        let template_content = fs::read_to_string(template_path)
            .expect("Failed to read email template file");
        #[derive(Serialize)]
        struct EmailContext {
            code: String,
        }
        let context = EmailContext {
            code: code.clone(),
        };

        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("email_template", &template_content)
            .expect("Failed to register template");
        let rendered_body = handlebars
            .render("email_template", &context)
            .expect("Failed to render template");

        let email = Message::builder()
            .from("hello@vhennus.com".parse().unwrap())
            .to(reciever_email.parse()?)
            .subject("Reset Password Email")
            .header(ContentType::TEXT_HTML)
            .body(rendered_body)
            .unwrap();

        match mailer.send(email).await {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("Failed to send email: {}", e);
                Err(Box::new(e))
            }
        }
    }

    pub async  fn send_ref_reminder_email(reciever_email:String)->Result<(), Box<dyn Error>>{
        let mailer:AsyncSmtpTransport<Tokio1Executor> = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.zoho.com")
            .unwrap()
            .credentials(Credentials::new(CONFIG.email.clone(), CONFIG.email_password.clone()))
            .build();
        let template_path = "templates/ref_reminder.hbs";
        let template_content = fs::read_to_string(template_path)
            .expect("Failed to read email template file");
        #[derive(Serialize)]
        struct EmailContext {

        }
        let context = EmailContext {

        };

        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("email_template", &template_content)
            .expect("Failed to register template");
        let rendered_body = handlebars
            .render("email_template", &context)
            .expect("Failed to render template");

        let email = Message::builder()
            .from("Vhennus <hello@vhennus.com>".parse().unwrap())
            .to(reciever_email.parse().unwrap())
            .subject("Referral Reminder")
            .header(ContentType::TEXT_HTML)
            .body(rendered_body)
            .unwrap();

        match mailer.send(email).await {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("Failed to send email: {}", e);
                Err(Box::new(e))
            }
        }
    }

}
