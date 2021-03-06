use crate::scene::{
    SetColliderCollisionGroupsCommand, SetColliderIsSensorCommand, SetColliderPositionCommand,
    SetColliderRotationCommand,
};
use crate::sidebar::{make_bool_input_field, make_int_input_field, make_vec3_input_field};
use crate::{
    gui::{BuildContext, Ui, UiMessage, UiNode},
    physics::Collider,
    scene::{SceneCommand, SetColliderFrictionCommand, SetColliderRestitutionCommand},
    send_sync_message,
    sidebar::{make_f32_input_field, make_text_mark, COLUMN_WIDTH, ROW_HEIGHT},
    Message,
};
use rg3d::core::math::{quat_from_euler, RotationOrder, UnitQuaternionExt};
use rg3d::gui::message::{CheckBoxMessage, Vec3EditorMessage};
use rg3d::{
    core::algebra::Vector3,
    core::pool::Handle,
    gui::{
        grid::{Column, GridBuilder, Row},
        message::{MessageDirection, NumericUpDownMessage, UiMessageData},
        widget::WidgetBuilder,
    },
};
use std::sync::mpsc::Sender;

pub struct ColliderSection {
    pub section: Handle<UiNode>,
    friction: Handle<UiNode>,
    restitution: Handle<UiNode>,
    position: Handle<UiNode>,
    rotation: Handle<UiNode>,
    collision_groups: Handle<UiNode>,
    collision_mask: Handle<UiNode>,
    is_sensor: Handle<UiNode>,
    sender: Sender<Message>,
}

impl ColliderSection {
    pub fn new(ctx: &mut BuildContext, sender: Sender<Message>) -> Self {
        let friction;
        let restitution;
        let position;
        let rotation;
        let collision_groups;
        let collision_mask;
        let is_sensor;
        let section = GridBuilder::new(
            WidgetBuilder::new()
                .with_child(make_text_mark(ctx, "Friction", 0))
                .with_child({
                    friction = make_f32_input_field(ctx, 0, 0.0, std::f32::MAX, 0.1);
                    friction
                })
                .with_child(make_text_mark(ctx, "Restitution", 1))
                .with_child({
                    restitution = make_f32_input_field(ctx, 1, 0.0, std::f32::MAX, 0.1);
                    restitution
                })
                .with_child(make_text_mark(ctx, "Collider Position", 2))
                .with_child({
                    position = make_vec3_input_field(ctx, 2);
                    position
                })
                .with_child(make_text_mark(ctx, "Collider Rotation", 3))
                .with_child({
                    rotation = make_vec3_input_field(ctx, 3);
                    rotation
                })
                .with_child(make_text_mark(ctx, "Collision Groups", 4))
                .with_child({
                    collision_groups = make_int_input_field(ctx, 4, 0, u16::MAX as i32, 1);
                    collision_groups
                })
                .with_child(make_text_mark(ctx, "Collision Mask", 5))
                .with_child({
                    collision_mask = make_int_input_field(ctx, 5, 0, u16::MAX as i32, 1);
                    collision_mask
                })
                .with_child(make_text_mark(ctx, "Is Sensor", 6))
                .with_child({
                    is_sensor = make_bool_input_field(ctx, 6);
                    is_sensor
                }),
        )
        .add_column(Column::strict(COLUMN_WIDTH))
        .add_column(Column::stretch())
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .build(ctx);

        Self {
            section,
            sender,
            friction,
            restitution,
            position,
            rotation,
            is_sensor,
            collision_mask,
            collision_groups,
        }
    }

    pub fn sync_to_model(&mut self, collider: &Collider, ui: &mut Ui) {
        send_sync_message(
            ui,
            NumericUpDownMessage::value(
                self.friction,
                MessageDirection::ToWidget,
                collider.friction,
            ),
        );

        send_sync_message(
            ui,
            NumericUpDownMessage::value(
                self.restitution,
                MessageDirection::ToWidget,
                collider.restitution,
            ),
        );

        send_sync_message(
            ui,
            Vec3EditorMessage::value(
                self.position,
                MessageDirection::ToWidget,
                collider.translation,
            ),
        );

        let euler = collider.rotation.to_euler();
        let euler_degrees = Vector3::new(
            euler.x.to_degrees(),
            euler.y.to_degrees(),
            euler.z.to_degrees(),
        );
        send_sync_message(
            ui,
            Vec3EditorMessage::value(self.rotation, MessageDirection::ToWidget, euler_degrees),
        );

        send_sync_message(
            ui,
            CheckBoxMessage::checked(
                self.is_sensor,
                MessageDirection::ToWidget,
                Some(collider.is_sensor),
            ),
        );

        send_sync_message(
            ui,
            NumericUpDownMessage::value(
                self.collision_groups,
                MessageDirection::ToWidget,
                (collider.collision_groups >> 16) as f32,
            ),
        );

        send_sync_message(
            ui,
            NumericUpDownMessage::value(
                self.collision_mask,
                MessageDirection::ToWidget,
                (collider.collision_groups & 0x0000FFFF) as f32,
            ),
        );
    }

    pub fn handle_message(
        &mut self,
        message: &UiMessage,
        collider: &Collider,
        handle: Handle<Collider>,
    ) {
        if message.direction() == MessageDirection::FromWidget {
            match message.data() {
                &UiMessageData::NumericUpDown(NumericUpDownMessage::Value(value)) => {
                    if message.destination() == self.friction && collider.friction.ne(&value) {
                        self.sender
                            .send(Message::DoSceneCommand(SceneCommand::SetColliderFriction(
                                SetColliderFrictionCommand::new(handle, value),
                            )))
                            .unwrap();
                    } else if message.destination() == self.restitution
                        && collider.restitution.ne(&value)
                    {
                        self.sender
                            .send(Message::DoSceneCommand(
                                SceneCommand::SetColliderRestitution(
                                    SetColliderRestitutionCommand::new(handle, value),
                                ),
                            ))
                            .unwrap();
                    } else if message.destination() == self.collision_mask {
                        let mask = (collider.collision_groups & 0xFFFF0000) | value as u32;
                        self.sender
                            .send(Message::DoSceneCommand(
                                SceneCommand::SetColliderCollisionGroups(
                                    SetColliderCollisionGroupsCommand::new(handle, mask),
                                ),
                            ))
                            .unwrap();
                    } else if message.destination() == self.collision_groups {
                        let groups =
                            (collider.collision_groups & 0x0000FFFF) | ((value as u32) << 16);
                        self.sender
                            .send(Message::DoSceneCommand(
                                SceneCommand::SetColliderCollisionGroups(
                                    SetColliderCollisionGroupsCommand::new(handle, groups),
                                ),
                            ))
                            .unwrap();
                    }
                }
                UiMessageData::Vec3Editor(Vec3EditorMessage::Value(value)) => {
                    if message.destination() == self.position && collider.translation.ne(value) {
                        self.sender
                            .send(Message::DoSceneCommand(SceneCommand::SetColliderPosition(
                                SetColliderPositionCommand::new(handle, *value),
                            )))
                            .unwrap();
                    } else if message.destination() == self.rotation {
                        let old_rotation = collider.rotation;
                        let euler = Vector3::new(
                            value.x.to_radians(),
                            value.y.to_radians(),
                            value.z.to_radians(),
                        );
                        let new_rotation = quat_from_euler(euler, RotationOrder::XYZ);
                        if !old_rotation.approx_eq(&new_rotation, 0.00001) {
                            self.sender
                                .send(Message::DoSceneCommand(SceneCommand::SetColliderRotation(
                                    SetColliderRotationCommand::new(handle, new_rotation),
                                )))
                                .unwrap();
                        }
                    }
                }
                UiMessageData::CheckBox(CheckBoxMessage::Check(checked)) => {
                    if message.destination() == self.is_sensor {
                        let value = checked.unwrap_or_default();
                        if value != collider.is_sensor {
                            self.sender
                                .send(Message::DoSceneCommand(SceneCommand::SetColliderIsSensor(
                                    SetColliderIsSensorCommand::new(handle, value),
                                )))
                                .unwrap();
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
