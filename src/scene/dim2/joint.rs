//! Joint is used to restrict motion of two rigid bodies.

use crate::utils::log::Log;
use crate::{
    core::{
        algebra::{UnitComplex, Vector2},
        inspect::{Inspect, PropertyInfo},
        pool::Handle,
        visitor::prelude::*,
    },
    engine::resource_manager::ResourceManager,
    impl_directly_inheritable_entity_trait,
    scene::{
        base::{Base, BaseBuilder},
        graph::Graph,
        node::Node,
        variable::{InheritError, TemplateVariable},
        DirectlyInheritableEntity,
    },
};
use fxhash::FxHashMap;
use rapier2d::dynamics::JointHandle;
use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
};

/// Ball joint locks any translational moves between two objects on the axis between objects, but
/// allows rigid bodies to perform relative rotations. The real world example is a human shoulder,
/// pendulum, etc.
#[derive(Clone, Debug, Visit, PartialEq, Inspect)]
pub struct BallJoint {
    /// Where the prismatic joint is attached on the first body, expressed in the local space of the
    /// first attached body.
    pub local_anchor1: Vector2<f32>,
    /// Where the prismatic joint is attached on the second body, expressed in the local space of the
    /// second attached body.
    pub local_anchor2: Vector2<f32>,
    /// Are the limits enabled for this joint?
    pub limits_enabled: bool,
    /// The axis of the limit cone for this joint, if the local-space of the first body.
    pub limits_local_axis1: Vector2<f32>,
    /// The axis of the limit cone for this joint, if the local-space of the first body.
    pub limits_local_axis2: Vector2<f32>,
    /// The maximum angle allowed between the two limit axes in world-space.
    pub limits_angle: f32,
}

impl Default for BallJoint {
    fn default() -> Self {
        Self {
            local_anchor1: Default::default(),
            local_anchor2: Default::default(),
            limits_enabled: false,
            limits_local_axis1: Default::default(),
            limits_local_axis2: Default::default(),
            limits_angle: f32::MAX,
        }
    }
}

/// A fixed joint ensures that two rigid bodies does not move relative to each other. There is no
/// straightforward real-world example, but it can be thought as two bodies were "welded" together.
#[derive(Clone, Debug, Visit, PartialEq, Inspect)]
pub struct FixedJoint {
    /// Local translation for the first body.
    pub local_anchor1_translation: Vector2<f32>,
    /// Local rotation for the first body.
    pub local_anchor1_rotation: UnitComplex<f32>,
    /// Local translation for the second body.
    pub local_anchor2_translation: Vector2<f32>,
    /// Local rotation for the second body.
    pub local_anchor2_rotation: UnitComplex<f32>,
}

impl Default for FixedJoint {
    fn default() -> Self {
        Self {
            local_anchor1_translation: Default::default(),
            local_anchor1_rotation: UnitComplex::new(0.0),
            local_anchor2_translation: Default::default(),
            local_anchor2_rotation: UnitComplex::new(0.0),
        }
    }
}

/// Prismatic joint prevents any relative movement between two rigid-bodies, except for relative
/// translations along one axis. The real world example is a sliders that used to support drawers.
#[derive(Clone, Debug, Visit, PartialEq, Inspect)]
pub struct PrismaticJoint {
    /// Where the prismatic joint is attached on the first body, expressed in the local space of the
    /// first attached body.
    pub local_anchor1: Vector2<f32>,
    /// The rotation axis of this revolute joint expressed in the local space of the first attached
    /// body.
    pub local_axis1: Vector2<f32>,
    /// Where the prismatic joint is attached on the second body, expressed in the local space of the
    /// second attached body.
    pub local_anchor2: Vector2<f32>,
    /// The rotation axis of this revolute joint expressed in the local space of the second attached
    /// body.
    pub local_axis2: Vector2<f32>,
    /// Whether or not this joint should enforce translational limits along its axis.
    pub limits_enabled: bool,
    /// The min an max relative position of the attached bodies along this joint's axis.
    pub limits: [f32; 2],
}

impl Default for PrismaticJoint {
    fn default() -> Self {
        Self {
            local_anchor1: Default::default(),
            local_axis1: Vector2::y(),
            local_anchor2: Default::default(),
            local_axis2: Vector2::x(),
            limits_enabled: false,
            limits: [f32::MIN, f32::MAX],
        }
    }
}

/// The exact kind of the joint.
#[derive(Clone, Debug, PartialEq, Visit)]
pub enum JointParams {
    /// See [`BallJoint`] for more info.
    BallJoint(BallJoint),
    /// See [`FixedJoint`] for more info.
    FixedJoint(FixedJoint),
    /// See [`PrismaticJoint`] for more info.
    PrismaticJoint(PrismaticJoint),
}

impl Inspect for JointParams {
    fn properties(&self) -> Vec<PropertyInfo<'_>> {
        match self {
            JointParams::BallJoint(v) => v.properties(),
            JointParams::FixedJoint(v) => v.properties(),
            JointParams::PrismaticJoint(v) => v.properties(),
        }
    }
}

impl Default for JointParams {
    fn default() -> Self {
        Self::BallJoint(Default::default())
    }
}

/// Joint is used to restrict motion of two rigid bodies. There are numerous examples of joints in
/// real life: door hinge, ball joints in human arms, etc.
#[derive(Visit, Inspect, Debug)]
pub struct Joint {
    base: Base,

    #[inspect(getter = "Deref::deref")]
    pub(crate) params: TemplateVariable<JointParams>,

    #[inspect(getter = "Deref::deref")]
    pub(crate) body1: TemplateVariable<Handle<Node>>,

    #[inspect(getter = "Deref::deref")]
    pub(crate) body2: TemplateVariable<Handle<Node>>,

    #[visit(skip)]
    #[inspect(skip)]
    pub(crate) native: Cell<JointHandle>,
}

impl_directly_inheritable_entity_trait!(Joint;
    params,
    body1,
    body2
);

impl Default for Joint {
    fn default() -> Self {
        Self {
            base: Default::default(),
            params: Default::default(),
            body1: Default::default(),
            body2: Default::default(),
            native: Cell::new(JointHandle::invalid()),
        }
    }
}

impl Deref for Joint {
    type Target = Base;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for Joint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Joint {
    /// Creates a raw copy of the joint node. This is for internal use only!
    pub fn raw_copy(&self) -> Self {
        Self {
            base: self.base.raw_copy(),
            params: self.params.clone(),
            body1: self.body1.clone(),
            body2: self.body2.clone(),
            native: Cell::new(JointHandle::invalid()),
        }
    }

    /// Returns a shared reference to the current joint parameters.
    pub fn params(&self) -> &JointParams {
        &self.params
    }

    /// Returns a mutable reference to the current joint parameters. Obtaining the mutable reference
    /// will force the engine to do additional calculations to reflect changes to the physics engine.
    pub fn params_mut(&mut self) -> &mut JointParams {
        self.params.get_mut()
    }

    /// Sets the first body of the joint. The handle should point to the RigidBody node, otherwise
    /// the joint will have no effect!
    pub fn set_body1(&mut self, handle: Handle<Node>) {
        self.body1.set(handle);
    }

    /// Returns current first body of the joint.
    pub fn body1(&self) -> Handle<Node> {
        *self.body1
    }

    /// Sets the second body of the joint. The handle should point to the RigidBody node, otherwise
    /// the joint will have no effect!
    pub fn set_body2(&mut self, handle: Handle<Node>) {
        self.body2.set(handle);
    }

    /// Returns current second body of the joint.
    pub fn body2(&self) -> Handle<Node> {
        *self.body2
    }

    pub(crate) fn restore_resources(&mut self, _resource_manager: ResourceManager) {}

    // Prefab inheritance resolving.
    pub(crate) fn inherit(&mut self, parent: &Node) -> Result<(), InheritError> {
        self.base.inherit_properties(parent)?;
        if let Node::Joint2D(parent) = parent {
            self.try_inherit_self_properties(parent)?;
        }
        Ok(())
    }

    pub(crate) fn reset_inheritable_properties(&mut self) {
        self.base.reset_inheritable_properties();
        self.reset_self_inheritable_properties();
    }

    pub(crate) fn remap_handles(
        &mut self,
        old_new_mapping: &FxHashMap<Handle<Node>, Handle<Node>>,
    ) {
        self.base.remap_handles(old_new_mapping);

        if let Some(entry) = old_new_mapping.get(&self.body1()) {
            self.body1.set_silent(*entry);
        } else {
            Log::warn(format!(
                "Unable to remap first body of a joint {}. Handle is {}!",
                self.name(),
                self.body1()
            ))
        }

        if let Some(entry) = old_new_mapping.get(&self.body2()) {
            self.body2.set_silent(*entry);
        } else {
            Log::warn(format!(
                "Unable to remap second body of a joint {}. Handle is {}!",
                self.name(),
                self.body2()
            ))
        }
    }
}

/// Joint builder allows you to build Joint node in a declarative manner.
pub struct JointBuilder {
    base_builder: BaseBuilder,
    params: JointParams,
    body1: Handle<Node>,
    body2: Handle<Node>,
}

impl JointBuilder {
    /// Creates a new joint builder instance.
    pub fn new(base_builder: BaseBuilder) -> Self {
        Self {
            base_builder,
            params: Default::default(),
            body1: Default::default(),
            body2: Default::default(),
        }
    }

    /// Sets desired joint parameters which defines exact type of the joint.
    pub fn with_params(mut self, params: JointParams) -> Self {
        self.params = params;
        self
    }

    /// Sets desired first body of the joint. This handle should be a handle to rigid body node,
    /// otherwise joint will have no effect!
    pub fn with_body1(mut self, body1: Handle<Node>) -> Self {
        self.body1 = body1;
        self
    }

    /// Sets desired second body of the joint. This handle should be a handle to rigid body node,
    /// otherwise joint will have no effect!
    pub fn with_body2(mut self, body2: Handle<Node>) -> Self {
        self.body2 = body2;
        self
    }

    /// Creates new Joint node, but does not add it to the graph.
    pub fn build_joint(self) -> Joint {
        Joint {
            base: self.base_builder.build_base(),
            params: self.params.into(),
            body1: self.body1.into(),
            body2: self.body2.into(),
            native: Cell::new(JointHandle::invalid()),
        }
    }

    /// Creates new Joint node, but does not add it to the graph.
    pub fn build_node(self) -> Node {
        Node::Joint2D(self.build_joint())
    }

    /// Creates new Joint node and adds it to the graph.
    pub fn build(self, graph: &mut Graph) -> Handle<Node> {
        graph.add_node(self.build_node())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        core::algebra::Vector2,
        scene::{
            base::{test::check_inheritable_properties_equality, BaseBuilder},
            dim2::joint::{BallJoint, JointBuilder, JointParams},
            node::Node,
        },
    };

    #[test]
    fn test_joint_2d_inheritance() {
        let parent = JointBuilder::new(BaseBuilder::new())
            .with_params(JointParams::BallJoint(BallJoint {
                local_anchor1: Vector2::new(1.0, 0.0),
                local_anchor2: Vector2::new(1.0, 1.0),
                limits_enabled: true,
                limits_local_axis1: Vector2::new(1.0, 1.0),
                limits_local_axis2: Vector2::new(1.0, 1.0),
                limits_angle: 1.57,
            }))
            .build_node();

        let mut child = JointBuilder::new(BaseBuilder::new()).build_joint();

        child.inherit(&parent).unwrap();

        if let Node::Joint2D(parent) = parent {
            check_inheritable_properties_equality(&child.base, &parent.base);
            check_inheritable_properties_equality(&child, &parent);
        } else {
            unreachable!();
        }
    }
}
