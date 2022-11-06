use rand::{thread_rng, Rng};

pub fn get_message_id() -> Vec<u8> {
    (0..16)
        .map(|_x| thread_rng().gen::<u8>())
        .collect::<Vec<u8>>()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[actix_web::test]
    async fn test_get_message_id_exaclty_sixteen_sym() {
        let parse_result = get_message_id();

        assert_eq!(parse_result.len(), 16);
    }

    #[actix_web::test]
    async fn test_get_message_id_twice_random() {
        let parse_result_1 = get_message_id();
        let parse_result_2 = get_message_id();

        assert!(parse_result_1 != parse_result_2);
    }
}
