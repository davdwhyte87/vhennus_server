
pub struct  Formatter{

}
impl Formatter {
    pub fn response_formatter(code:String, message:String, data:String)->String{
        return format!("{}{}{}{}{}{}",code,"\n",message,"\n",data,"\n");
    }

    pub fn request_formatter(
        action:String, data:String, 
        message_singnature:String, 
        sender_node_public_key:String,
        is_braodcasted:String
    )->String{
        return format!(
            "{}{}{}{}{}{}{}{}{}{}",
            action,
            "\n",
            data,
            "\n",
            message_singnature,
            "\n",
            sender_node_public_key,
            "\n",
            is_braodcasted,
            "\n"
            );
    }
}
