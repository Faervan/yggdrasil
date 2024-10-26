use std::f32::consts::PI;

use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};

use super::components::GameComponentParent;


pub fn setup_light(
    mut commands: Commands,
    mut ambient_light: ResMut<AmbientLight>,
) {
   ambient_light.brightness = 150.;
   commands.spawn((
       DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: light_consts::lux::OVERCAST_DAY,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_rotation_x(-PI / 4.),
                ..default()
            },
            // The default cascade config is designed to handle large scenes.
            // As this example has a much smaller world, we can tighten the shadow
            // bounds for better visual quality.
            cascade_shadow_config: CascadeShadowConfigBuilder {
                first_cascade_far_bound: 4.0,
                maximum_distance: 200.0,
                ..default()
            }
            .into(),
            ..default()
           },
            GameComponentParent {},
    ));
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                color: Color::WHITE,
                shadows_enabled: true,
                intensity: 100000000.,
                range: 200.,
                ..default()
            },
            transform: Transform::from_xyz(0., 50., 0.),
            ..default()
        },
        GameComponentParent {},
    ));
}
