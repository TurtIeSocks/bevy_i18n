use std::path::Path;

use bevy::{
    app::{Plugin, PreStartup, Update},
    asset::{AssetServer, Handle},
    ecs::{
        query::With,
        schedule::{
            common_conditions::{resource_changed, resource_exists},
            IntoSystemConfigs,
        },
        system::{Commands, Query, Res, ResMut},
    },
    prelude::resource_removed,
    text::{Font, TextFont},
    ui::widget::Text,
};

use crate::{
    components::I18nText,
    prelude::I18nFont,
    resources::{FontFolder, FontManager, FontsLoading, I18n},
};

include!(concat!(env!("OUT_DIR"), "/bevy_simple_i18n.rs"));

/// Initializes the `bevy_simple_i18n` plugin
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_simple_i18n::prelude::*;
///
/// fn main() {
///     App::new()
///         .add_plugins(I18nPlugin)
///         .run();
/// }
/// ```
pub struct I18nPlugin;

impl Plugin for I18nPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<I18n>()
            .init_resource::<FontManager>()
            .init_resource::<FontsLoading>()
            .add_systems(PreStartup, load_dynamic_fonts)
            .add_systems(
                Update,
                (
                    monitor_font_loading.run_if(resource_exists::<FontsLoading>),
                    update_translations.run_if(resource_removed::<FontsLoading>),
                    update_translations.run_if(resource_changed::<I18n>),
                ),
            );
    }
}

/// Loads the dynamic fonts specified in the [FONT_FAMILIES] constant that's generated by the build script
///
/// TODO: Make the loading state more controllable
fn load_dynamic_fonts(
    mut font_manager: ResMut<FontManager>,
    asset_server: Res<bevy::asset::AssetServer>,
) {
    for dyn_font in FONT_FAMILIES.iter() {
        bevy::log::debug!("Loading dynamic font family: {}", dyn_font.family);
        let mut font_folder = FontFolder::default();
        font_folder.fallback = asset_server.load(Path::new(dyn_font.path).join("fallback.ttf"));
        for font in dyn_font.locales.iter() {
            bevy::log::debug!("Loading font: {}", font);
            let locale = font.split('.').next().expect("Locale is required");
            let path = Path::new(dyn_font.path).join(font);
            let handler: Handle<Font> = asset_server.load(path);
            font_folder.fonts.insert(locale.to_string(), handler);
        }
        font_manager.insert(dyn_font.family.to_string(), font_folder);
    }
}

/// Monitors the font loading state and removes the [FontsLoading] resource when all fonts are loaded
///
/// TODO: Make the loading state more controllable
fn monitor_font_loading(
    mut commands: Commands,
    font_manager: Res<FontManager>,
    asset_server: Res<AssetServer>,
) {
    for folder in font_manager.fonts.values() {
        for font in folder.fonts.values() {
            if !asset_server.is_loaded(font.id()) {
                return;
            }
        }
    }
    commands.remove_resource::<FontsLoading>();
    bevy::log::debug!("All fonts loaded");
}

/// Auto updates the translations for the text entities that have the [I18nText] component
/// whenever the [I18n] resource changes
fn update_translations(
    font_manager: bevy::ecs::system::Res<FontManager>,
    mut text_query: Query<(&mut Text, &mut TextFont, Option<&I18nFont>, &I18nText), With<I18nText>>,
) {
    bevy::log::debug!("Updating translations");
    for (mut text, mut text_font, dyn_font, key) in text_query.iter_mut() {
        text.0 = key.translate();
        if let Some(dyn_font) = dyn_font {
            text_font.font = font_manager.get(&dyn_font.0, key.locale.clone());
        }
    }
}
