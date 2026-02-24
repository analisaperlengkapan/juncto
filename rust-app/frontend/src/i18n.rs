use leptos::*;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Locale {
    #[default]
    En,
    #[allow(dead_code)]
    Id,
}

#[derive(Clone)]
pub struct I18nContext {
    pub locale: RwSignal<Locale>,
}

pub fn provide_i18n_context() {
    let locale = create_rw_signal(Locale::default());
    provide_context(I18nContext { locale });
}

#[allow(dead_code)]
pub fn use_i18n() -> I18nContext {
    use_context::<I18nContext>().expect("I18nContext must be provided")
}

pub fn t(key: &str) -> String {
    let ctx = use_context::<I18nContext>();
    let locale = ctx.map(|c| c.locale.get()).unwrap_or_default();

    translate(locale, key)
}

fn translate(locale: Locale, key: &str) -> String {
    match (locale, key) {
        (Locale::En, "settings") => "Settings".to_string(),
        (Locale::Id, "settings") => "Pengaturan".to_string(),
        (Locale::En, "profile") => "Profile".to_string(),
        (Locale::Id, "profile") => "Profil".to_string(),
        (Locale::En, "devices") => "Devices".to_string(),
        (Locale::Id, "devices") => "Perangkat".to_string(),
        (Locale::En, "display_name") => "Display Name".to_string(),
        (Locale::Id, "display_name") => "Nama Tampilan".to_string(),
        (Locale::En, "save_profile") => "Save Profile".to_string(),
        (Locale::Id, "save_profile") => "Simpan Profil".to_string(),
        (Locale::En, "camera") => "Camera".to_string(),
        (Locale::Id, "camera") => "Kamera".to_string(),
        (Locale::En, "video_quality") => "Video Quality".to_string(),
        (Locale::Id, "video_quality") => "Kualitas Video".to_string(),
        (Locale::En, "microphone") => "Microphone".to_string(),
        (Locale::Id, "microphone") => "Mikrofon".to_string(),
        (Locale::En, "default") => "Default".to_string(),
        (Locale::Id, "default") => "Bawaan".to_string(),
        (Locale::En, "preview_only") => "This is a local preview only.".to_string(),
        (Locale::Id, "preview_only") => "Ini hanya pratinjau lokal.".to_string(),
        (Locale::En, "color") => "Color: ".to_string(),
        (Locale::Id, "color") => "Warna: ".to_string(),
        (Locale::En, "virtual_background") => "Virtual Background".to_string(),
        (Locale::Id, "virtual_background") => "Latar Belakang Virtual".to_string(),
        (Locale::En, "none") => "None".to_string(),
        (Locale::Id, "none") => "Tidak Ada".to_string(),
        (Locale::En, "blur") => "Blur".to_string(),
        (Locale::Id, "blur") => "Buram".to_string(),
        (Locale::En, "image") => "Image".to_string(),
        (Locale::Id, "image") => "Gambar".to_string(),
        (Locale::En, "done") => "Done".to_string(),
        (Locale::Id, "done") => "Selesai".to_string(),
        (Locale::En, "share_video") => "Share Video".to_string(),
        (Locale::Id, "share_video") => "Bagikan Video".to_string(),
        (Locale::En, "youtube_url") => "YouTube URL".to_string(),
        (Locale::Id, "youtube_url") => "URL YouTube".to_string(),
        (Locale::En, "cancel") => "Cancel".to_string(),
        (Locale::Id, "cancel") => "Batal".to_string(),
        (Locale::En, "share") => "Share".to_string(),
        (Locale::Id, "share") => "Bagikan".to_string(),
        (Locale::En, "apply_devices") => "Apply Devices".to_string(),
        (Locale::Id, "apply_devices") => "Terapkan Perangkat".to_string(),
        (Locale::En, "invite_people") => "Invite People".to_string(),
        (Locale::Id, "invite_people") => "Undang Orang".to_string(),
        (Locale::En, "copy_link") => "Copy Link".to_string(),
        (Locale::Id, "copy_link") => "Salin Tautan".to_string(),
        (Locale::En, "link_copied") => "Link Copied!".to_string(),
        (Locale::Id, "link_copied") => "Tautan Disalin!".to_string(),
        (Locale::En, "share_link_hint") => "Share this link with others to invite them to the meeting.".to_string(),
        (Locale::Id, "share_link_hint") => "Bagikan tautan ini dengan orang lain untuk mengundang mereka.".to_string(),
        (Locale::En, "chat_panel") => "Chat".to_string(),
        (Locale::Id, "chat_panel") => "Obrolan".to_string(),
         // Add more keys as needed
        (_, k) => k.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation() {
        assert_eq!(translate(Locale::En, "settings"), "Settings");
        assert_eq!(translate(Locale::Id, "settings"), "Pengaturan");
        assert_eq!(translate(Locale::En, "unknown"), "unknown");
    }
}
