/// Animation blending state machine.
///
/// Machine is used to blend multiple animation as well as perform automatic "smooth transition
/// between states. Let have a quick look at simple machine graph:
///
///                                                  +-------------+
///                                                  |  Idle Anim  |
///                                                  +------+------+
///                                                         |
///           Walk Weight                                   |
/// +-----------+      +-------+           Walk->Idle Rule  |
/// | Walk Anim +------+       |                            |
/// +-----------+      |       |      +-------+         +---+---+
///                    | Blend |      |       +-------->+       |
///                    |       +------+ Walk  |         |  Idle |
/// +-----------+      |       |      |       +<--------+       |
/// | Aim Anim  +------+       |      +--+----+         +---+---+
/// +-----------+      +-------+         |                  ^
///           Aim Weight                 | Idle->Walk Rule  |
///                                      |                  |
///                       Walk->Run Rule |    +---------+   | Run->Idle Rule
///                                      |    |         |   |
///                                      +--->+   Run   +---+
///                                           |         |
///                                           +----+----+
///                                                |
///                                                |
///                                         +------+------+
///                                         |  Run Anim   |
///                                         +-------------+
///
/// Here we have Walk, Idle, Run states which uses different sources of poses:
/// - Walk - is most complicated here - it uses result of blending between
///   Aim and Walk animations with different weights. This is useful if your
///   character can only walk or can walk *and* aim at the same time. Desired pose
///   determined by Walk Weight and Aim Weight parameters combination.
/// - Run and idle both directly uses animation as pose source.
///
/// There are four transitions between three states each with its own rule. Rule
/// is just Rule parameter which can have boolean value that indicates that transition
/// should be activated.
///
/// Example
///
/// ```no_run
/// use rg3d::{
///     animation::machine::{
///         Machine, State, Transition, PoseNode, BlendPose,
///         Parameter, PlayAnimation, PoseWeight, BlendAnimation
///     },
///     core::pool::Handle
/// };
///
/// // Assume that these are correct handles.
/// let idle_animation = Handle::default();
/// let walk_animation = Handle::default();
/// let aim_animation = Handle::default();
///
/// let mut machine = Machine::new();
///
/// machine.add_parameter("WalkToIdle", Parameter::Rule(false));
/// machine.add_parameter("IdleToWalk", Parameter::Rule(false));
///
/// let aim = machine.add_node(PoseNode::PlayAnimation(PlayAnimation::new(aim_animation)));
/// let walk = machine.add_node(PoseNode::PlayAnimation(PlayAnimation::new(walk_animation)));
///
/// // Blend two animations together
/// let blend_aim_walk = machine.add_node(PoseNode::BlendAnimations(
/// 	BlendAnimation::new(vec![
/// 		BlendPose::new(PoseWeight::Constant(0.75), aim),
/// 		BlendPose::new(PoseWeight::Constant(0.25), walk)
/// 	])
/// ));
///
/// let walk_state = machine.add_state(State::new("Walk", blend_aim_walk));
///
/// let idle = machine.add_node(PoseNode::PlayAnimation(PlayAnimation::new(idle_animation)));
/// let idle_state = machine.add_state(State::new("Idle", idle));
///
/// machine.add_transition(Transition::new("Walk->Idle", walk_state, idle_state, 1.0, "WalkToIdle"));
/// machine.add_transition(Transition::new("Idle->Walk", idle_state, walk_state, 1.0, "IdleToWalk"));
///
/// ```

use crate::{
    animation::{
        Animation,
        AnimationContainer,
        AnimationPose,
    },
    core::{
        pool::Pool,
        pool::Handle,
        visitor::{
            Visit,
            Visitor,
            VisitError,
            VisitResult,
        },
    },
};
use std::{
    cell::{RefCell, Ref},
    collections::{
        HashMap,
        VecDeque,
    },
};

pub enum Event {
    StateEnter(Handle<State>),
    StateLeave(Handle<State>),
    ActiveStateChanged(Handle<State>),
}

#[derive(Default)]
pub struct PlayAnimation {
    animation: Handle<Animation>,
    output_pose: RefCell<AnimationPose>,
}

impl PlayAnimation {
    pub fn new(animation: Handle<Animation>) -> Self {
        Self {
            animation,
            output_pose: Default::default(),
        }
    }
}

impl Visit for PlayAnimation {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        visitor.enter_region(name)?;

        self.animation.visit("Animation", visitor)?;

        visitor.leave_region()
    }
}

pub enum Parameter {
    Weight(f32),
    Rule(bool),
}

impl Default for Parameter {
    fn default() -> Self {
        Parameter::Weight(0.0)
    }
}

impl Parameter {
    fn from_id(id: i32) -> Result<Self, String> {
        match id {
            0 => Ok(Parameter::Weight(0.0)),
            1 => Ok(Parameter::Rule(false)),
            _ => Err(format!("Invalid parameter id {}", id))
        }
    }

    fn id(&self) -> i32 {
        match self {
            Parameter::Weight(_) => 0,
            Parameter::Rule(_) => 1,
        }
    }
}

impl Visit for Parameter {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        visitor.enter_region(name)?;

        let mut id = self.id();
        id.visit("Id", visitor)?;
        if visitor.is_reading() {
            *self = Self::from_id(id)?;
        }

        match self {
            Parameter::Weight(weight) => weight.visit("Value", visitor)?,
            Parameter::Rule(rule) => rule.visit("Value", visitor)?,
        }

        visitor.leave_region()
    }
}

pub enum PoseWeight {
    Constant(f32),
    Parameter(String),
}

impl Default for PoseWeight {
    fn default() -> Self {
        PoseWeight::Constant(0.0)
    }
}

impl PoseWeight {
    fn from_id(id: i32) -> Result<Self, String> {
        match id {
            0 => Ok(PoseWeight::Parameter(Default::default())),
            1 => Ok(PoseWeight::Constant(0.0)),
            _ => Err(format!("Invalid pose weight id {}", id))
        }
    }

    fn id(&self) -> i32 {
        match self {
            PoseWeight::Constant(_) => 0,
            PoseWeight::Parameter(_) => 1,
        }
    }
}

impl Visit for PoseWeight {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        visitor.enter_region(name)?;

        let mut id = self.id();
        id.visit("Id", visitor)?;
        if visitor.is_reading() {
            *self = Self::from_id(id)?;
        }

        match self {
            PoseWeight::Constant(constant) => constant.visit("Value", visitor)?,
            PoseWeight::Parameter(param_id) => param_id.visit("ParamId", visitor)?,
        }

        visitor.leave_region()
    }
}

#[derive(Default)]
pub struct BlendPose {
    weight: PoseWeight,
    pose_source: Handle<PoseNode>,
}

impl BlendPose {
    pub fn new(weight: PoseWeight, pose_source: Handle<PoseNode>) -> Self {
        Self {
            weight,
            pose_source,
        }
    }
}

impl Visit for BlendPose {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        visitor.enter_region(name)?;

        self.weight.visit("Weight", visitor)?;
        self.pose_source.visit("PoseSource", visitor)?;

        visitor.leave_region()
    }
}

#[derive(Default)]
pub struct BlendAnimation {
    pose_sources: RefCell<Vec<BlendPose>>,
    output_pose: RefCell<AnimationPose>,
}

impl BlendAnimation {
    pub fn new(poses: Vec<BlendPose>) -> Self {
        Self {
            pose_sources: RefCell::new(poses),
            output_pose: Default::default(),
        }
    }
}

impl Visit for BlendAnimation {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> Result<(), VisitError> {
        visitor.enter_region(name)?;

        self.pose_sources.visit("PoseSources", visitor)?;

        visitor.leave_region()
    }
}

pub enum PoseNode {
    PlayAnimation(PlayAnimation),
    BlendAnimations(BlendAnimation),
}

impl Default for PoseNode {
    fn default() -> Self {
        PoseNode::PlayAnimation(Default::default())
    }
}

impl PoseNode {
    fn from_id(id: i32) -> Result<Self, String> {
        match id {
            0 => Ok(PoseNode::PlayAnimation(Default::default())),
            1 => Ok(PoseNode::BlendAnimations(Default::default())),
            _ => Err(format!("Invalid pose node id {}", id))
        }
    }

    fn id(&self) -> i32 {
        match self {
            PoseNode::PlayAnimation(_) => 0,
            PoseNode::BlendAnimations(_) => 1,
        }
    }
}

macro_rules! dispatch {
    ($self:ident, $func:ident, $($args:expr),*) => {
        match $self {
            PoseNode::PlayAnimation(v) => v.$func($($args),*),
            PoseNode::BlendAnimations(v) => v.$func($($args),*),
        }
    };
}

impl Visit for PoseNode {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        let mut kind_id = self.id();
        kind_id.visit("KindId", visitor)?;
        if visitor.is_reading() {
            *self = PoseNode::from_id(kind_id)?;
        }

        dispatch!(self, visit, name, visitor)
    }
}

#[derive(Default)]
pub struct State {
    name: String,
    root: Handle<PoseNode>,
    pose: AnimationPose,
}

pub type ParameterContainer = HashMap<String, Parameter>;

trait EvaluatePose {
    fn eval_pose(&self, nodes: &Pool<PoseNode>, params: &ParameterContainer, animations: &AnimationContainer) -> Ref<AnimationPose>;
}

impl EvaluatePose for PlayAnimation {
    fn eval_pose(&self, _nodes: &Pool<PoseNode>, _params: &ParameterContainer, animations: &AnimationContainer) -> Ref<AnimationPose> {
        animations.get(self.animation)
            .get_pose()
            .clone_into(&mut self.output_pose.borrow_mut());
        self.output_pose.borrow()
    }
}

impl EvaluatePose for BlendAnimation {
    fn eval_pose(&self, nodes: &Pool<PoseNode>, params: &ParameterContainer, animations: &AnimationContainer) -> Ref<AnimationPose> {
        self.output_pose.borrow_mut().reset();
        for blend_pose in self.pose_sources.borrow_mut().iter_mut() {
            let weight = match blend_pose.weight {
                PoseWeight::Constant(value) => value,
                PoseWeight::Parameter(ref param_id) => {
                    if let Some(param) = params.get(param_id) {
                        if let Parameter::Weight(weight) = param {
                            *weight
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    }
                }
            };

            let pose_source = nodes.borrow(blend_pose.pose_source).eval_pose(nodes, params, animations);
            self.output_pose.borrow_mut().blend_with(&pose_source, weight);
        }
        self.output_pose.borrow()
    }
}

impl EvaluatePose for PoseNode {
    fn eval_pose(&self, nodes: &Pool<PoseNode>, params: &ParameterContainer, animations: &AnimationContainer) -> Ref<AnimationPose> {
        dispatch!(self, eval_pose, nodes, params, animations)
    }
}

impl State {
    pub fn new(name: &str, root: Handle<PoseNode>) -> Self {
        Self {
            name: name.to_owned(),
            root,
            pose: Default::default(),
        }
    }

    fn update(&mut self, nodes: &Pool<PoseNode>, params: &ParameterContainer, animations: &AnimationContainer) {
        self.pose.reset();
        nodes.borrow(self.root)
            .eval_pose(nodes, params, animations)
            .clone_into(&mut self.pose);
    }
}

impl Visit for State {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        visitor.enter_region(name)?;

        self.name.visit("Name", visitor)?;
        self.root.visit("Root", visitor)?;

        visitor.leave_region()
    }
}

#[derive(Default)]
pub struct Transition {
    name: String,
    /// Total amount of time to transition from `src` to `dst` state.
    transition_time: f32,
    elapsed_time: f32,
    src: Handle<State>,
    dest: Handle<State>,
    /// Identifier of Rule parameter which defines is transition should be activated or not.
    rule: String,
    /// 0 - evaluates `src` pose, 1 - `dest`, 0..1 - blends `src` and `dest`
    blend_factor: f32,
}

impl Visit for Transition {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        visitor.enter_region(name)?;

        self.name.visit("Name", visitor)?;
        self.transition_time.visit("TransitionTime", visitor)?;
        self.elapsed_time.visit("ElapsedTime", visitor)?;
        self.src.visit("Source", visitor)?;
        self.dest.visit("Dest", visitor)?;
        self.rule.visit("Rule", visitor)?;
        self.blend_factor.visit("BlendFactor", visitor)?;

        visitor.leave_region()
    }
}

impl Transition {
    pub fn new(name: &str, src: Handle<State>, dest: Handle<State>, time: f32, rule: &str) -> Transition {
        Self {
            name: name.to_owned(),
            transition_time: time,
            elapsed_time: 0.0,
            src,
            dest,
            rule: rule.to_owned(),
            blend_factor: 0.0,
        }
    }

    fn reset(&mut self) {
        self.elapsed_time = 0.0;
        self.blend_factor = 0.0;
    }

    fn update(&mut self, dt: f32) {
        self.elapsed_time += dt;
        if self.elapsed_time > self.transition_time {
            self.elapsed_time = self.transition_time;
        }
        self.blend_factor = self.elapsed_time / self.transition_time;
    }

    fn is_done(&self) -> bool {
        self.transition_time == self.elapsed_time
    }
}

#[derive(Default)]
pub struct Machine {
    nodes: Pool<PoseNode>,
    states: Pool<State>,
    transitions: Pool<Transition>,
    final_pose: AnimationPose,
    active_state: Handle<State>,
    active_transition: Handle<Transition>,
    parameters: ParameterContainer,
    events: VecDeque<Event>,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            states: Default::default(),
            transitions: Default::default(),
            final_pose: Default::default(),
            active_state: Default::default(),
            active_transition: Default::default(),
            parameters: Default::default(),
            events: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: PoseNode) -> Handle<PoseNode> {
        self.nodes.spawn(node)
    }

    pub fn add_parameter(&mut self, id: &str, parameter: Parameter) {
        self.parameters.insert(id.to_owned(), parameter);
    }

    pub fn set_parameter(&mut self, id: &str, parameter: Parameter) {
        if let Some(param) = self.parameters.get_mut(id) {
            *param = parameter;
        }
    }

    pub fn add_state(&mut self, state: State) -> Handle<State> {
        self.states.spawn(state)
    }

    pub fn add_transition(&mut self, transition: Transition) {
        let _ = self.transitions.spawn(transition);
    }

    pub fn get_state(&self, state: Handle<State>) -> Option<&State> {
        self.states.try_borrow(state)
    }

    fn push_event(&mut self, event: Event) {
        if self.events.len() < 2048 {
            self.events.push_back(event)
        }
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    pub fn evaluate_pose(&mut self, animations: &AnimationContainer, dt: f32) -> &AnimationPose {
        self.final_pose.reset();

        // Gather actual poses for each state.
        for state in self.states.iter_mut() {
            state.update(&self.nodes, &self.parameters, animations);
        }

        if self.active_transition.is_none() {
            // Find transition.
            for (handle, transition) in self.transitions.pair_iter_mut() {
                if transition.dest == self.active_state {
                    continue;
                }
                if let Some(rule) = self.parameters.get(&transition.rule) {
                    if let Parameter::Rule(active) = rule {
                        if *active {
                            self.active_transition = handle;
                            self.active_state = transition.dest;
                            break;
                        }
                    } else {
                        // TODO: Assert?
                    }
                }
            }
        }

        // Double check for active transition because we can have empty machine.
        if self.active_transition.is_some() {
            let transition = self.transitions.borrow_mut(self.active_transition);

            // Blend between source and dest states.
            self.final_pose.blend_with(&self.states.borrow_mut(transition.src).pose, 1.0 - transition.blend_factor);
            self.final_pose.blend_with(&self.states.borrow_mut(transition.dest).pose, transition.blend_factor);

            transition.update(dt);

            if transition.is_done() {
                transition.reset();
                self.active_transition = Handle::NONE;
            }
        } else if self.active_state.is_some() {
            let state = self.states.borrow_mut(self.active_state);

            // Just get pose from active state.
            state.pose.clone_into(&mut self.final_pose);
        }

        &self.final_pose
    }
}

impl Visit for Machine {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        visitor.enter_region(name)?;

        self.parameters.visit("Parameters", visitor)?;
        self.nodes.visit("Nodes", visitor)?;
        self.transitions.visit("Transitions", visitor)?;
        self.states.visit("States", visitor)?;
        self.active_state.visit("ActiveState", visitor)?;
        self.active_transition.visit("ActiveTransition", visitor)?;

        visitor.leave_region()
    }
}

pub struct MachineBuilder {}

