pub struct ClientSecondMessage {
    salted_password: Vec<u8>,
    auth_message: String,
    password: Vec<u8>
}

impl ClientSecondMessage {
    pub(crate) fn new(p0: Vec<u8>, p1: String, p2: Vec<u8>) -> ClientSecondMessage {
        ClientSecondMessage {
            salted_password: p0,
            auth_message: p1,
            password: p2
        }
    }

    pub fn get_salted_password(&self) -> &Vec<u8> {
        &self.salted_password
    }

    pub fn get_auth_message(&self) -> &String {
        &self.auth_message
    }
    pub fn get_password(&self) -> &Vec<u8> {
        &self.password
    }

}