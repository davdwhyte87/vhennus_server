use handlebars::Handlebars;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::header;
use serde_json::json;
use crate::models::helper::EmailData;


pub const ACTIVATE_EMAIL:&str ="\
<h1>Activation</h1>
<p>This is your activation code {}
";
pub fn send_email(email_data:EmailData)->bool{
    let email = Message::builder()
        .from("HDOS <team@hdos.org>".parse().unwrap())
        .reply_to("HDOS <team@hdos.org>".parse().unwrap())
        .header(header::ContentType::TEXT_HTML)
        .to(email_data.to.parse().unwrap())
        .subject(email_data.subject)
        .body(email_data.body)
        .unwrap();

    let creds = Credentials::new("vhenngames@gmail.com".to_string(), "grmkcavvcvjxpfzu".to_string());

// Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

// Send the email
    match mailer.send(&email) {
        Ok(_) =>{
            println!("Email sent successfully!");
            return true;
        } ,
        Err(e) => {
            println!("Could not send email: {:?}", e);
            return false;
        },
    }

}

pub fn get_body(json_values:serde_json::Value)->String{
    let mut reg = Handlebars::new();
    reg.register_template_file ("email", "html/activate_new_account.hbs").unwrap();
    let order_email_content = reg.render ("email", &serde_json::json!(json_values)).unwrap();
    return order_email_content
}