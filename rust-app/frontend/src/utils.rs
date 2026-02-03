pub fn create_room_url(name: &str) -> String {
    let encoded_name = urlencoding::encode(name);
    format!("/room/{}", encoded_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_room_url() {
        assert_eq!(create_room_url("My Room"), "/room/My%20Room");
        assert_eq!(create_room_url("Test"), "/room/Test");
    }
}
