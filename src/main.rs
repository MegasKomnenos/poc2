mod misc;
mod state;
mod system;
mod component;
mod asset;
mod ai;

use crate::misc::*;
use crate::state::*;
use crate::system::*;

extern crate ron;
extern crate enum_primitive_derive;
extern crate num_traits;

use amethyst::{
    core::transform::TransformBundle,
    prelude::*,
    input::{ InputBundle, StringBindings, },
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
};
use amethyst_tiles::{
    MortonEncoder2D, 
    RenderTiles2D,
};

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let asset_dir = app_root.join("asset");
    let config_dir = app_root.join("config");
    let display_config_path = config_dir.join("display.ron");
    let input_config_path = config_dir.join("input.ron");

    let game_data = GameDataBuilder::default()
        .with(SystemCameraMovement::default(), "Camera Movement System", &[])
        .with(SystemColorMap::default(), "Map Coloring System", &[])
        .with(SystemMovement::default(), "Character Movement System", &[])
        .with(SystemSpawnChar::default(), "Character Spawning System", &[])
        .with(SystemSetMoveGoal::default(), "Movement Goal System", &[])
        .with_bundle(
            InputBundle::<StringBindings>::new()
                .with_bindings_from_file(input_config_path)?,
        )?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderTiles2D::<MiscTile, MortonEncoder2D, MiscTileBounds>::default())
        )?
        .with_bundle(TransformBundle::new())?;

    let mut game = Application::new(asset_dir, PocLoad::default(), game_data)?;
    game.run();

    Ok(())
}
