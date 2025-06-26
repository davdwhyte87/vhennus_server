use std::error::Error;
use std::fs;
use awc::Client;
use base64::Engine;
use base64::engine::general_purpose;
use handlebars::Handlebars;
use lettre::transport::smtp::authentication::Credentials;

use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use lettre::message::header::ContentType;
use log::{debug, error};
use serde_derive::{Deserialize, Serialize};
use crate::CONFIG;
use crate::models::app_error::AppError;

pub struct EmailService{
    api_url: String,
    client: reqwest::Client,
}
#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
}

#[derive(Debug, Serialize)]
struct SendEmailRequest {
    email: EmailDetails,
}

#[derive(Debug, Serialize)]
struct EmailDetails {
    subject: String,
    from: SenderInfo,
    to: Vec<RecipientInfo>,
    html: String,
    text: String,
    auto_plain_text: bool
}
#[derive(Debug, Serialize)]
struct RecipientInfo {
    email: String,
}

#[derive(Debug, Serialize)]
struct SenderInfo {
    name: String,
    email: String,
}

impl EmailService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            api_url: "https://api.sendpulse.com".to_string(),
        }
    }

    async fn get_access_token(&self) -> Result<String, reqwest::Error> {
        let response = self.client
            .post(&format!("{}/oauth/access_token", self.api_url))
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &*CONFIG.send_plus_client_id),
                ("client_secret", &*CONFIG.send_plus_client_secrete),
            ])
            .send()
            .await?
            .json::<AccessTokenResponse>()
            .await?;

        Ok(format!("Bearer {}", response.access_token))
    }
    pub async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        html_body: &str,
    ) -> Result<(), reqwest::Error> {
        let auth_token = self.get_access_token().await?;
        let from_email = "team@vhennus.com";
        let from_name = "Vhennus";

        let email_request = SendEmailRequest {
            email: EmailDetails {
                subject: subject.to_string(),
                from: SenderInfo {
                    name: from_name.to_string(),
                    email: from_email.to_string(),
                },
                to: vec![RecipientInfo {
                    email: to_email.to_string(),
                }],
                html:  general_purpose::STANDARD.encode(html_body),
                text: html_body.to_string(),
                auto_plain_text: true
            },
        };
        //debug!("mail html: {}", html_body);
        self.client
            .post(&format!("{}/smtp/emails", self.api_url))
            .header("Authorization", auth_token)
            .json(&email_request)
            .send()
            .await?;

        Ok(())
    }

    pub async fn send_test_email(&self) -> Result<(), AppError> {
        match Self::send_email(
            self,
            "kingstonwhyte87@gmail.com",
            "Donnu", "<h1>kingstonwhyte8787</h1>",).await{
            Ok(_) => Ok(()),
            Err(e) => {
                error!("{}", e);
                return Err(AppError::SendMailError)
            }
        }
    }

    pub async  fn send_signup_email2(&self, reciever_email:String, code:String)->Result<(), AppError>{

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

        match Self::send_email(&self, &reciever_email,"Vhennus Signup", &rendered_body).await{
            Ok(_) => Ok(()),
            Err(err) => {
                error!("{}", err);
                Err(AppError::SendMailError)
            }
        }
    }

    pub async  fn send_reset_password_email2(&self, reciever_email:String, code:String)->Result<(), AppError>{

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

        match Self::send_email(&self, &reciever_email,"Vhennus Reset Password", &rendered_body).await{
            Ok(_) => Ok(()),
            Err(err) => {
                error!("{}", err);
                Err(AppError::SendMailError)
            }
        }
    }



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

    pub async  fn send_ref_reminder_email(&self, reciever_email:String)->Result<(), AppError>{
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

        match Self::send_email(&self, &reciever_email,"Referral Reminder", &rendered_body).await{
            Ok(_) => Ok(()),
            Err(err) => {
                error!("{}", err);
                Err(AppError::SendMailError)
            }
        }
    }

}
