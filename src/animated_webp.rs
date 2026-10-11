// Encodage d'un WebP animé, image par image, avec libwebp.
//
// La crate `webp` garde toutes les images en mémoire et perd la durée de la dernière : on
// appelle donc directement l'encodeur d'animation de libwebp.
use std::ffi::{CStr, c_int};
use std::mem::MaybeUninit;
use std::ptr;

use libwebp_sys as sys;

pub struct AnimatedWebpEncoder {
    // Invariant : pointeur non nul rendu par `WebPAnimEncoderNewInternal`, libéré dans `drop`
    encoder: *mut sys::WebPAnimEncoder,
    config: sys::WebPConfig,
    width: u32,
    height: u32,
}

impl AnimatedWebpEncoder {
    /// `loop_count` vaut 0 pour une boucle infinie.
    pub fn new(width: u32, height: u32, quality: f32, loop_count: i32) -> Result<Self, String> {
        let mut config =
            sys::WebPConfig::new().map_err(|()| "configuration WebP invalide".to_owned())?;
        config.quality = quality;

        let abi_version = sys::WebPGetMuxABIVersion();
        let mut options = MaybeUninit::<sys::WebPAnimEncoderOptions>::uninit();
        // SAFETY: `options` pointe sur une zone de la bonne taille, que libwebp initialise
        // entièrement quand l'appel réussit ; on ne la lit que dans ce cas.
        let mut options = unsafe {
            if sys::WebPAnimEncoderOptionsInitInternal(options.as_mut_ptr(), abi_version) == 0 {
                return Err("version de libwebp incompatible".to_owned());
            }
            options.assume_init()
        };
        options.anim_params.loop_count = loop_count;

        // SAFETY: `options` est initialisée et reste valide pendant l'appel, qui la copie.
        let encoder = unsafe {
            sys::WebPAnimEncoderNewInternal(width as c_int, height as c_int, &options, abi_version)
        };
        if encoder.is_null() {
            return Err("impossible de créer l'encodeur d'animation".to_owned());
        }
        Ok(Self {
            encoder,
            config,
            width,
            height,
        })
    }

    /// Ajoute une image RGBA aux dimensions de l'animation, affichée à partir de `timestamp_ms`.
    pub fn add_frame(&mut self, rgba: &[u8], timestamp_ms: i32) -> Result<(), String> {
        let stride = self.width as usize * 4;
        if rgba.len() != stride * self.height as usize {
            return Err("image aux dimensions inattendues".to_owned());
        }

        let mut picture =
            sys::WebPPicture::new().map_err(|()| "version de libwebp incompatible".to_owned())?;
        picture.use_argb = 1;
        picture.width = self.width as c_int;
        picture.height = self.height as c_int;

        // SAFETY: `rgba` contient exactement `height` lignes de `stride` octets (vérifié
        // ci-dessus). L'import copie les pixels dans `picture`, l'ajout copie `picture` dans
        // l'encodeur : on peut libérer `picture` dans tous les cas juste après.
        let is_added = unsafe {
            let is_added = sys::WebPPictureImportRGBA(&mut picture, rgba.as_ptr(), stride as c_int)
                != 0
                && sys::WebPAnimEncoderAdd(self.encoder, &mut picture, timestamp_ms, &self.config)
                    != 0;
            sys::WebPPictureFree(&mut picture);
            is_added
        };
        if is_added {
            Ok(())
        } else {
            Err(self.last_error())
        }
    }

    /// Termine l'animation. `end_timestamp_ms` fixe la durée de la dernière image.
    pub fn finish(self, end_timestamp_ms: i32) -> Result<Vec<u8>, String> {
        let mut data = sys::WebPData {
            bytes: ptr::null(),
            size: 0,
        };
        // SAFETY: une image nulle signale la fin de l'animation à libwebp. `data` reçoit un
        // tampon alloué par libwebp, copié puis libéré par `WebPDataClear` avant de sortir.
        unsafe {
            if sys::WebPAnimEncoderAdd(self.encoder, ptr::null_mut(), end_timestamp_ms, ptr::null())
                == 0
                || sys::WebPAnimEncoderAssemble(self.encoder, &mut data) == 0
            {
                return Err(self.last_error());
            }
            let bytes = if data.bytes.is_null() {
                Vec::new()
            } else {
                std::slice::from_raw_parts(data.bytes, data.size).to_vec()
            };
            sys::WebPDataClear(&mut data);
            Ok(bytes)
        }
    }

    fn last_error(&self) -> String {
        // SAFETY: libwebp renvoie une chaîne C terminée par un zéro, valide tant que
        // l'encodeur existe ; elle est copiée avant de rendre la main.
        unsafe {
            let message = sys::WebPAnimEncoderGetError(self.encoder);
            if message.is_null() {
                "erreur inconnue".to_owned()
            } else {
                CStr::from_ptr(message).to_string_lossy().into_owned()
            }
        }
    }
}

impl Drop for AnimatedWebpEncoder {
    fn drop(&mut self) {
        // SAFETY: `encoder` est non nul et n'est libéré qu'ici.
        unsafe { sys::WebPAnimEncoderDelete(self.encoder) }
    }
}
