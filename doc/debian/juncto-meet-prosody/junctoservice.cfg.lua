-- Enables dial-in for Juncto meet components customers
VirtualHost "jungasi.meet.juncto"
    modules_enabled = {
        "ping";
        "bosh";
        "muc_password_check";
    }
    authentication = "token"
    app_id = "juncto";
    asap_key_server = "https://jaas-public-keys.juncto.net/juncto-components/prod-Juncto"
    asap_accepted_issuers = { "jaas-components" }
    asap_accepted_audiences = { "jungasi.junctomeet.example.com" }
