pub mod draw;

use crate::{
    utils::{
        pool::{Pool, Handle},
        rcpool::RcHandle,
        UnsafeCollectionView,
    },
    math::{
        vec2::Vec2,
        Rect,
    },
    gui::draw::{
        Color,
        DrawingContext,
        FormattedText,
        CommandKind,
        FormattedTextBuilder,
    },
    resource::{
        Resource,
        ttf::Font,
    },
    math,
};
use glutin::{
    VirtualKeyCode,
    MouseButton,
    WindowEvent,
    ElementState,
};
use std::collections::VecDeque;
use std::cell::{Cell, RefCell};
use std::any::{TypeId, Any};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum HorizontalAlignment {
    Stretch,
    Left,
    Center,
    Right,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum VerticalAlignment {
    Stretch,
    Top,
    Center,
    Bottom,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Thickness {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Thickness {
    pub fn zero() -> Thickness {
        Thickness {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        }
    }

    pub fn uniform(v: f32) -> Thickness {
        Thickness {
            left: v,
            top: v,
            right: v,
            bottom: v,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Visibility {
    Visible,
    Collapsed,
    Hidden,
}

#[derive(Debug)]
pub struct Text {
    owner_handle: Handle<UINode>,
    need_update: bool,
    text: String,
    font: Handle<Font>,
    vertical_alignment: VerticalAlignment,
    horizontal_alignment: HorizontalAlignment,
    formatted_text: Option<FormattedText>,
}

impl Drawable for Text {
    fn draw(&mut self, drawing_context: &mut DrawingContext, font_cache: &Pool<Font>, bounds: &Rect<f32>, color: Color) {
        if self.need_update {
            if let Some(font) = font_cache.borrow(&self.font) {
                let formatted_text = FormattedTextBuilder::reuse(self.formatted_text.take().unwrap())
                    .with_size(Vec2::make(bounds.w, bounds.h))
                    .with_font(font)
                    .with_text(self.text.as_str())
                    .with_color(color)
                    .with_horizontal_alignment(self.horizontal_alignment)
                    .with_vertical_alignment(self.vertical_alignment)
                    .build();
                self.formatted_text = Some(formatted_text);
            }
            self.need_update = true; // TODO
        }
        drawing_context.draw_text(Vec2::make(bounds.x, bounds.y), self.formatted_text.as_ref().unwrap());
    }
}

impl Text {
    pub fn new() -> Text {
        Text {
            owner_handle: Handle::none(),
            text: String::new(),
            need_update: true,
            vertical_alignment: VerticalAlignment::Top,
            horizontal_alignment: HorizontalAlignment::Left,
            formatted_text: Some(FormattedTextBuilder::new().build()),
            font: Handle::none(),
        }
    }

    pub fn set_text(&mut self, text: &str) -> &mut Self {
        self.text.clear();
        self.text += text;
        self.need_update = true;
        self
    }

    pub fn get_text(&self) -> &str {
        self.text.as_str()
    }

    pub fn set_font(&mut self, font: Handle<Font>) -> &mut Self {
        self.font = font;
        self.need_update = true;
        self
    }

    pub fn set_vertical_alignment(&mut self, valign: VerticalAlignment) -> &mut Self {
        self.vertical_alignment = valign;
        self
    }

    pub fn set_horizontal_alignment(&mut self, halign: HorizontalAlignment) -> &mut Self {
        self.horizontal_alignment = halign;
        self
    }
}

pub struct CommonBuilderFields {
    name: Option<String>,
    width: Option<f32>,
    height: Option<f32>,
    desired_position: Option<Vec2>,
    vertical_alignment: Option<VerticalAlignment>,
    horizontal_alignment: Option<HorizontalAlignment>,
    max_size: Option<Vec2>,
    min_size: Option<Vec2>,
    color: Option<Color>,
    row: Option<usize>,
    column: Option<usize>,
    margin: Option<Thickness>,
    event_handlers: Option<RoutedEventHandlerList>,
    children: Vec<Handle<UINode>>,
}

impl CommonBuilderFields {
    pub fn new() -> Self {
        Self {
            name: None,
            width: None,
            height: None,
            vertical_alignment: None,
            horizontal_alignment: None,
            max_size: None,
            min_size: None,
            color: None,
            row: None,
            column: None,
            margin: None,
            desired_position: None,
            event_handlers: Some(Default::default()),
            children: Vec::new(),
        }
    }

    pub fn apply(&mut self, ui: &mut UserInterface, node_handle: &Handle<UINode>) {
        if let Some(node) = ui.nodes.borrow_mut(node_handle) {
            if let Some(width) = self.width {
                node.width.set(width);
            }
            if let Some(height) = self.height {
                node.height.set(height);
            }
            if let Some(valign) = self.vertical_alignment {
                node.vertical_alignment = valign;
            }
            if let Some(halign) = self.horizontal_alignment {
                node.horizontal_alignment = halign;
            }
            if let Some(max_size) = self.max_size {
                node.max_size = max_size;
            }
            if let Some(min_size) = self.min_size {
                node.min_size = min_size;
            }
            if let Some(color) = self.color {
                node.color = color;
            }
            if let Some(row) = self.row {
                node.row = row;
            }
            if let Some(column) = self.column {
                node.column = column;
            }
            if let Some(margin) = self.margin {
                node.margin = margin;
            }
            if let Some(desired_position) = self.desired_position {
                node.desired_local_position.set(desired_position);
            }
            if self.event_handlers.is_some() {
                node.event_handlers = self.event_handlers.take().unwrap();
            }
            if let Some(name) = self.name.take() {
                node.name = name;
            }
        }
        for child_handle in self.children.iter() {
            ui.link_nodes(child_handle, node_handle);
        }
    }
}

macro_rules! impl_default_builder_methods {
    () => (
        pub fn with_width(mut self, width: f32) -> Self {
            self.common.width = Some(width);
            self
        }

        pub fn with_height(mut self, height: f32) -> Self {
            self.common.height = Some(height);
            self
        }

        pub fn with_vertical_alignment(mut self, valign: VerticalAlignment) -> Self {
            self.common.vertical_alignment = Some(valign);
            self
        }

        pub fn with_horizontal_alignment(mut self, halign: HorizontalAlignment) -> Self {
            self.common.horizontal_alignment = Some(halign);
            self
        }

        pub fn with_max_size(mut self, max_size: Vec2) -> Self {
            self.common.max_size = Some(max_size);
            self
        }

        pub fn with_min_size(mut self, min_size: Vec2) -> Self {
            self.common.min_size = Some(min_size);
            self
        }

        pub fn with_color(mut self, color: Color) -> Self {
            self.common.color = Some(color);
            self
        }

        pub fn on_row(mut self, row: usize) -> Self {
            self.common.row = Some(row);
            self
        }

        pub fn on_column(mut self, column: usize) -> Self {
            self.common.column = Some(column);
            self
        }

        pub fn with_margin(mut self, margin: Thickness) -> Self {
            self.common.margin = Some(margin);
            self
        }

        pub fn with_desired_position(mut self, desired_position: Vec2) -> Self {
            self.common.desired_position = Some(desired_position);
            self
        }

        pub fn with_child(mut self, handle: Handle<UINode>) -> Self {
            if handle.is_some() {
                self.common.children.push(handle);
            }
            self
        }

        pub fn with_name(mut self, name: &str) -> Self {
            self.common.name = Some(String::from(name));
            self
        }

        pub fn with_handler(mut self, handler_type: RoutedEventHandlerType, handler: Box<RoutedEventHandler>) -> Self {
            if let Some(ref mut handlers) = self.common.event_handlers {
                handlers[handler_type as usize] = Some(handler);
            }
            self
        }
    )
}

pub struct GenericNodeBuilder {
    kind: UINodeKind,
    common: CommonBuilderFields,
}

impl GenericNodeBuilder {
    pub fn new(kind: UINodeKind, common: CommonBuilderFields) -> Self {
        Self {
            kind,
            common,
        }
    }

    impl_default_builder_methods!();

    pub fn build(mut self, ui: &mut UserInterface) -> Handle<UINode> {
        let handle = ui.add_node(UINode::new(self.kind));
        self.common.apply(ui, &handle);
        handle
    }
}

pub struct CanvasBuilder {
    common: CommonBuilderFields
}

impl CanvasBuilder {
    pub fn new() -> Self {
        Self {
            common: CommonBuilderFields::new()
        }
    }

    impl_default_builder_methods!();

    pub fn build(self, ui: &mut UserInterface) -> Handle<UINode> {
        GenericNodeBuilder::new(UINodeKind::Canvas(Canvas::new()), self.common).build(ui)
    }
}

pub struct TextBuilder {
    text: Option<String>,
    font: Option<Handle<Font>>,
    common: CommonBuilderFields,
    vertical_text_alignment: Option<VerticalAlignment>,
    horizontal_text_alignment: Option<HorizontalAlignment>,
}

impl TextBuilder {
    pub fn new() -> Self {
        Self {
            text: None,
            font: None,
            vertical_text_alignment: None,
            horizontal_text_alignment: None,
            common: CommonBuilderFields::new(),
        }
    }

    impl_default_builder_methods!();

    pub fn with_text(mut self, text: &str) -> Self {
        self.text = Some(text.to_owned());
        self
    }

    pub fn with_font(mut self, font: Handle<Font>) -> Self {
        self.font = Some(font);
        self
    }

    pub fn build(mut self, ui: &mut UserInterface) -> Handle<UINode> {
        let mut text = Text::new();
        if let Some(font) = self.font {
            text.set_font(font.clone());
        } else {
            text.set_font(ui.default_font.clone());
        }
        if let Some(txt) = self.text {
            text.set_text(txt.as_str());
        }
        if let Some(valign) = self.vertical_text_alignment {
            text.set_vertical_alignment(valign);
        }
        if let Some(halign) = self.horizontal_text_alignment {
            text.set_horizontal_alignment(halign);
        }
        let handle = ui.add_node(UINode::new(UINodeKind::Text(text)));
        self.common.apply(ui, &handle);
        handle
    }

    pub fn with_vertical_text_alignment(mut self, valign: VerticalAlignment) -> Self {
        self.vertical_text_alignment = Some(valign);
        self
    }

    pub fn with_horizontal_text_alignment(mut self, halign: HorizontalAlignment) -> Self {
        self.horizontal_text_alignment = Some(halign);
        self
    }
}

pub struct BorderBuilder {
    stroke_thickness: Option<Thickness>,
    stroke_color: Option<Color>,
    common: CommonBuilderFields,
}

impl BorderBuilder {
    pub fn new() -> Self {
        Self {
            stroke_color: None,
            stroke_thickness: None,
            common: CommonBuilderFields::new(),
        }
    }

    impl_default_builder_methods!();

    pub fn with_stroke_thickness(mut self, stroke_thickness: Thickness) -> Self {
        self.stroke_thickness = Some(stroke_thickness);
        self
    }

    pub fn with_stroke_color(mut self, color: Color) -> Self {
        self.stroke_color = Some(color);
        self
    }

    pub fn build(mut self, ui: &mut UserInterface) -> Handle<UINode> {
        let mut border = Border::new();
        if let Some(stroke_color) = self.stroke_color {
            border.stroke_color = stroke_color;
        }
        if let Some(stroke_thickness) = self.stroke_thickness {
            border.stroke_thickness = stroke_thickness;
        }
        let handle = ui.add_node(UINode::new(UINodeKind::Border(border)));
        self.common.apply(ui, &handle);
        handle
    }
}

trait Layout {
    fn measure_override(&self, ui: &UserInterface, available_size: &Vec2) -> Vec2;
    fn arrange_override(&self, ui: &UserInterface, final_size: &Vec2) -> Vec2;
}

trait Drawable {
    fn draw(&mut self, drawing_context: &mut DrawingContext, font_cache: &Pool<Font>, bounds: &Rect<f32>, color: Color);
}

#[derive(Debug)]
pub struct Border {
    owner_handle: Handle<UINode>,
    stroke_thickness: Thickness,
    stroke_color: Color,
}

impl Drawable for Border {
    fn draw(&mut self, drawing_context: &mut DrawingContext, _font_cache: &Pool<Font>, bounds: &Rect<f32>, color: Color) {
        drawing_context.push_rect_filled(&bounds, None, color);
        drawing_context.push_rect_vary(&bounds, self.stroke_thickness, self.stroke_color);
        drawing_context.commit(CommandKind::Geometry, 0);
    }
}

impl Layout for Border {
    fn measure_override(&self, ui: &UserInterface, available_size: &Vec2) -> Vec2 {
        let margin_x = self.stroke_thickness.left + self.stroke_thickness.right;
        let margin_y = self.stroke_thickness.top + self.stroke_thickness.bottom;

        let size_for_child = Vec2::make(
            available_size.x - margin_x,
            available_size.y - margin_y,
        );
        let mut desired_size = Vec2::new();

        if let Some(node) = ui.nodes.borrow(&self.owner_handle) {
            for child_handle in node.children.iter() {
                ui.measure(child_handle, &size_for_child);

                if let Some(child) = ui.nodes.borrow(child_handle) {
                    let child_desired_size = child.desired_size.get();
                    if child_desired_size.x > desired_size.x {
                        desired_size.x = child_desired_size.x;
                    }
                    if child_desired_size.y > desired_size.y {
                        desired_size.y = child_desired_size.y;
                    }
                }
            }
        }

        desired_size.x += margin_x;
        desired_size.y += margin_y;

        desired_size
    }

    fn arrange_override(&self, ui: &UserInterface, final_size: &Vec2) -> Vec2 {
        let rect_for_child = Rect::new(
            self.stroke_thickness.left, self.stroke_thickness.top,
            final_size.x - (self.stroke_thickness.right + self.stroke_thickness.left),
            final_size.y - (self.stroke_thickness.bottom + self.stroke_thickness.top),
        );

        if let Some(node) = ui.nodes.borrow(&self.owner_handle) {
            for child_handle in node.children.iter() {
                ui.arrange(child_handle, &rect_for_child);
            }
        }

        *final_size
    }
}

impl Border {
    pub fn new() -> Border {
        Border {
            owner_handle: Handle::none(),
            stroke_thickness: Thickness {
                left: 1.0,
                right: 1.0,
                top: 1.0,
                bottom: 1.0,
            },
            stroke_color: Color::white(),
        }
    }

    pub fn set_stroke_thickness(&mut self, thickness: Thickness) -> &mut Self {
        self.stroke_thickness = thickness;
        self
    }

    pub fn set_stroke_color(&mut self, color: Color) -> &mut Self {
        self.stroke_color = color;
        self
    }
}

pub struct Image {
    owner_handle: Handle<UINode>,
    texture: RcHandle<Resource>,
}

pub type ButtonClickEventHandler = dyn FnMut(&mut UserInterface, Handle<UINode>);

pub struct Button {
    owner_handle: Handle<UINode>,
    click: Option<Box<ButtonClickEventHandler>>,
}

impl Button {
    pub fn new() -> Self {
        Self {
            owner_handle: Handle::none(),
            click: None,
        }
    }

    pub fn set_on_click(&mut self, handler: Box<ButtonClickEventHandler>) {
        self.click = Some(handler);
    }
}

pub enum ButtonContent {
    Text(String),
    Node(Handle<UINode>),
}

pub struct ButtonBuilder {
    content: Option<ButtonContent>,
    click: Option<Box<ButtonClickEventHandler>>,
    common: CommonBuilderFields,
}

impl ButtonBuilder {
    pub fn new() -> Self {
        Self {
            content: None,
            click: None,
            common: CommonBuilderFields::new(),
        }
    }

    impl_default_builder_methods!();

    pub fn with_text(mut self, text: &str) -> Self {
        self.content = Some(ButtonContent::Text(text.to_owned()));
        self
    }

    pub fn with_node(mut self, node: Handle<UINode>) -> Self {
        self.content = Some(ButtonContent::Node(node));
        self
    }

    pub fn with_click(mut self, handler: Box<ButtonClickEventHandler>) -> Self {
        self.click = Some(handler);
        self
    }

    pub fn build(self, ui: &mut UserInterface) -> Handle<UINode> {
        let normal_color = Color::opaque(120, 120, 120);
        let pressed_color = Color::opaque(100, 100, 100);
        let hover_color = Color::opaque(160, 160, 160);

        let mut button = Button::new();
        button.click = self.click;

        GenericNodeBuilder::new(
            UINodeKind::Button(button), self.common)
            .with_handler(RoutedEventHandlerType::MouseDown, Box::new(move |ui, handle, _evt| {
                ui.capture_mouse(&handle);
            }))
            .with_handler(RoutedEventHandlerType::MouseUp, Box::new(move |ui, handle, evt| {
                // Take-Call-PutBack trick to bypass borrow checker
                let mut click_handler = None;

                if let Some(button_node) = ui.nodes.borrow_mut(&handle) {
                    if let UINodeKind::Button(button) = button_node.get_kind_mut() {
                        click_handler = button.click.take();
                    }
                }

                if let Some(ref mut handler) = click_handler {
                    handler(ui, handle.clone());
                    evt.handled = true;
                }

                // Second check required because event handler can remove node.
                if let Some(button_node) = ui.nodes.borrow_mut(&handle) {
                    if let UINodeKind::Button(button) = button_node.get_kind_mut() {
                        button.click = click_handler;
                    }
                }

                ui.release_mouse_capture();
            }))
            .with_child(BorderBuilder::new()
                .with_stroke_color(Color::opaque(200, 200, 200))
                .with_stroke_thickness(Thickness { left: 1.0, right: 1.0, top: 1.0, bottom: 1.0 })
                .with_color(normal_color)
                .with_handler(RoutedEventHandlerType::MouseEnter, Box::new(move |ui, handle, _evt| {
                    if let Some(back) = ui.nodes.borrow_mut(&handle) {
                        back.color = hover_color;
                    }
                }))
                .with_handler(RoutedEventHandlerType::MouseLeave, Box::new(move |ui, handle, _evt| {
                    if let Some(back) = ui.nodes.borrow_mut(&handle) {
                        back.color = normal_color;
                    }
                }))
                .with_handler(RoutedEventHandlerType::MouseDown, Box::new(move |ui, handle, _evt| {
                    if let Some(back) = ui.nodes.borrow_mut(&handle) {
                        back.color = pressed_color;
                    }
                }))
                .with_handler(RoutedEventHandlerType::MouseUp, Box::new(move |ui, handle, _evt| {
                    if let Some(back) = ui.nodes.borrow_mut(&handle) {
                        if back.is_mouse_over {
                            back.color = hover_color;
                        } else {
                            back.color = normal_color;
                        }
                    }
                }))
                .with_child(
                    if let Some(content) = self.content {
                        match content {
                            ButtonContent::Text(txt) => {
                                TextBuilder::new()
                                    .with_text(txt.as_str())
                                    .with_horizontal_text_alignment(HorizontalAlignment::Center)
                                    .with_vertical_text_alignment(VerticalAlignment::Center)
                                    .build(ui)
                            }
                            ButtonContent::Node(node) => node
                        }
                    } else {
                        Handle::none()
                    })
                .build(ui))
            .build(ui)
    }
}

pub struct ValueChangedArgs {
    source: Handle<UINode>,
    old_value: f32,
    new_value: f32,
}

pub type ValueChanged = dyn FnMut(&mut UserInterface, ValueChangedArgs);

pub struct ScrollBar {
    owner_handle: Handle<UINode>,
    min: f32,
    max: f32,
    value: f32,
    step: f32,
    orientation: Orientation,
    is_dragging: bool,
    offset: Vec2,
    value_changed: Option<Box<ValueChanged>>,
}

impl ScrollBar {
    pub const PART_CANVAS: &'static str = "PART_Canvas";
    pub const PART_INDICATOR: &'static str = "PART_Indicator";

    fn new() -> Self {
        Self {
            owner_handle: Handle::none(),
            min: 0.0,
            max: 100.0,
            value: 0.0,
            step: 1.0,
            orientation: Orientation::Horizontal,
            is_dragging: false,
            offset: Vec2::new(),
            value_changed: None,
        }
    }

    pub fn set_value(handle: &Handle<UINode>, ui: &mut UserInterface, value: f32) {
        let mut value_changed;
        let args;

        if let Some(node) = ui.nodes.borrow_mut(handle) {
            if let UINodeKind::ScrollBar(scroll_bar) = node.get_kind_mut() {
                let old_value = scroll_bar.value;
                let new_value = math::clampf(value, scroll_bar.min, scroll_bar.max);
                if new_value != old_value {
                    scroll_bar.value = new_value;
                    value_changed = scroll_bar.value_changed.take();
                    args = Some(ValueChangedArgs {
                        old_value,
                        new_value,
                        source: handle.clone(),
                    });
                } else {
                    return;
                }
            } else {
                return;
            }
        } else {
            return;
        }

        if let Some(ref mut handler) = value_changed {
            if let Some(args) = args {
                handler(ui, args)
            }
        }

        if let Some(node) = ui.nodes.borrow_mut(handle) {
            if let UINodeKind::ScrollBar(scroll_bar) = node.get_kind_mut() {
                scroll_bar.value_changed = value_changed;
            }
        }
    }

    pub fn set_max_value(handle: &Handle<UINode>, ui: &mut UserInterface, max: f32) {
        let mut new_value = None;
        if let Some(node) = ui.nodes.borrow_mut(handle) {
            if let UINodeKind::ScrollBar(scroll_bar) = node.get_kind_mut() {
                scroll_bar.max = max;
                if scroll_bar.max < scroll_bar.min {
                    std::mem::swap(&mut scroll_bar.min, &mut scroll_bar.max);
                }
                let old_value = scroll_bar.value;
                let clamped_new_value = math::clampf(scroll_bar.value, scroll_bar.min, scroll_bar.max);
                if clamped_new_value != old_value {
                    new_value = Some(clamped_new_value);
                }
            }
        }

        if let Some(new_value) = new_value {
            ScrollBar::set_value(handle, ui, new_value);
        }
    }

    pub fn set_min_value(handle: &Handle<UINode>, ui: &mut UserInterface, min: f32) {
        let mut new_value = None;
        if let Some(node) = ui.nodes.borrow_mut(handle) {
            if let UINodeKind::ScrollBar(scroll_bar) = node.get_kind_mut() {
                scroll_bar.min = min;
                if scroll_bar.min > scroll_bar.max {
                    std::mem::swap(&mut scroll_bar.min, &mut scroll_bar.max);
                }
                let old_value = scroll_bar.value;
                let clamped_new_value = math::clampf(scroll_bar.value, scroll_bar.min, scroll_bar.max);
                if clamped_new_value != old_value {
                    new_value = Some(clamped_new_value);
                }
            }
        }

        if let Some(new_value) = new_value {
            ScrollBar::set_value(handle, ui, new_value);
        }
    }
}

impl Layout for ScrollBar {
    fn measure_override(&self, ui: &UserInterface, available_size: &Vec2) -> Vec2 {
        ui.default_measure_override(&self.owner_handle, available_size)
    }

    fn arrange_override(&self, ui: &UserInterface, final_size: &Vec2) -> Vec2 {
        let size = ui.default_arrange_override(&self.owner_handle, final_size);


        // Adjust indicator position according to current value
        let percent = (self.value - self.min) / (self.max - self.min);

        let field_size = match ui.borrow_by_name_down(&self.owner_handle, Self::PART_CANVAS) {
            Some(canvas) => canvas.actual_size.get(),
            None => return size
        };

        if let Some(node) = ui.borrow_by_name_down(&self.owner_handle, Self::PART_INDICATOR) {
            match self.orientation {
                Orientation::Horizontal => {
                    node.set_desired_local_position(Vec2::make(
                        percent * maxf(0.0, field_size.x - node.actual_size.get().x),
                        0.0)
                    );
                    node.height.set(field_size.y);
                }
                Orientation::Vertical => {
                    node.set_desired_local_position(Vec2::make(
                        0.0,
                        percent * maxf(0.0, field_size.y - node.actual_size.get().y))
                    );
                    node.width.set(field_size.x);
                }
            }
        }

        size
    }
}

pub struct ScrollBarBuilder {
    min: Option<f32>,
    max: Option<f32>,
    value: Option<f32>,
    value_changed: Option<Box<ValueChanged>>,
    step: Option<f32>,
    orientation: Option<Orientation>,
    common: CommonBuilderFields,
}

#[derive(Copy, Clone, Debug)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

impl ScrollBarBuilder {
    pub fn new() -> Self {
        Self {
            min: None,
            max: None,
            value: None,
            step: None,
            value_changed: None,
            orientation: None,
            common: CommonBuilderFields::new(),
        }
    }

    impl_default_builder_methods!();

    pub fn with_min(mut self, min: f32) -> Self {
        self.min = Some(min);
        self
    }

    pub fn with_max(mut self, max: f32) -> Self {
        self.max = Some(max);
        self
    }

    pub fn with_value(mut self, value: f32) -> Self {
        self.value = Some(value);
        self
    }

    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    pub fn with_value_changed(mut self, value_changed: Box<ValueChanged>) -> Self {
        self.value_changed = Some(value_changed);
        self
    }

    pub fn build(self, ui: &mut UserInterface) -> Handle<UINode> {
        let mut scroll_bar = ScrollBar::new();
        if let Some(orientation) = self.orientation {
            scroll_bar.orientation = orientation;
        }
        scroll_bar.value_changed = self.value_changed;
        let orientation = scroll_bar.orientation;
        GenericNodeBuilder::new(UINodeKind::ScrollBar(scroll_bar), self.common)
            .with_child(BorderBuilder::new()
                .with_color(Color::opaque(120, 120, 120))
                .with_stroke_thickness(Thickness::uniform(1.0))
                .with_stroke_color(Color::opaque(200, 200, 200))
                .with_child(GridBuilder::new()
                    .add_rows(match orientation {
                        Orientation::Horizontal => vec![Row::stretch()],
                        Orientation::Vertical => vec![Row::auto(),
                                                      Row::stretch(),
                                                      Row::auto()]
                    })
                    .add_columns(match orientation {
                        Orientation::Horizontal => vec![Column::auto(),
                                                        Column::stretch(),
                                                        Column::auto()],
                        Orientation::Vertical => vec![Column::stretch()]
                    })
                    .with_child(ButtonBuilder::new()
                        .on_column(0)
                        .on_row(0)
                        .with_width(match orientation {
                            Orientation::Horizontal => 30.0,
                            Orientation::Vertical => std::f32::NAN
                        })
                        .with_height(match orientation {
                            Orientation::Horizontal => std::f32::NAN,
                            Orientation::Vertical => 30.0
                        })
                        .with_text(match orientation {
                            Orientation::Horizontal => "<",
                            Orientation::Vertical => "^"
                        })
                        .with_click(Box::new(move |ui, handle| {
                            let scroll_bar_handle = ui.find_by_criteria_up(&handle, |node| match node.kind {
                                UINodeKind::ScrollBar(..) => true,
                                _ => false
                            });

                            let new_value = if let Some(scroll_bar_node) = ui.nodes.borrow_mut(&scroll_bar_handle) {
                                if let UINodeKind::ScrollBar(scroll_bar) = scroll_bar_node.get_kind_mut() {
                                    scroll_bar.value - scroll_bar.step
                                } else {
                                    return;
                                }
                            } else {
                                return;
                            };

                            ScrollBar::set_value(&scroll_bar_handle, ui, new_value);
                        }))
                        .build(ui)
                    )
                    .with_child(CanvasBuilder::new()
                        .with_name(ScrollBar::PART_CANVAS)
                        .on_column(match orientation {
                            Orientation::Horizontal => 1,
                            Orientation::Vertical => 0
                        })
                        .on_row(match orientation {
                            Orientation::Horizontal => 0,
                            Orientation::Vertical => 1
                        })
                        .with_child(BorderBuilder::new()
                            .with_name(ScrollBar::PART_INDICATOR)
                            .with_stroke_color(Color::opaque(50, 50, 50))
                            .with_stroke_thickness(match orientation {
                                Orientation::Horizontal => Thickness { left: 1.0, top: 0.0, right: 1.0, bottom: 0.0 },
                                Orientation::Vertical => Thickness { left: 0.0, top: 1.0, right: 0.0, bottom: 1.0 }
                            })
                            .with_color(Color::opaque(255, 255, 255))
                            .with_width(30.0)
                            .with_height(30.0)
                            .with_handler(RoutedEventHandlerType::MouseDown, Box::new(move |ui, handle, evt| {
                                let indicator_pos = if let Some(node) = ui.nodes.borrow(&handle) {
                                    node.screen_position
                                } else {
                                    return;
                                };

                                if let RoutedEventKind::MouseDown { pos, .. } = evt.kind {
                                    if let Some(scroll_bar_node) = ui.borrow_by_criteria_up_mut(&handle, |node| match node.kind {
                                        UINodeKind::ScrollBar(..) => true,
                                        _ => false
                                    }) {
                                        if let UINodeKind::ScrollBar(scroll_bar) = scroll_bar_node.get_kind_mut() {
                                            scroll_bar.is_dragging = true;
                                            scroll_bar.offset = indicator_pos - pos;
                                        }
                                    }

                                    ui.capture_mouse(&handle);
                                    evt.handled = true;
                                }
                            }))
                            .with_handler(RoutedEventHandlerType::MouseUp, Box::new(move |ui, handle, evt| {
                                if let Some(scroll_bar_node) = ui.borrow_by_criteria_up_mut(&handle, |node| match node.kind {
                                    UINodeKind::ScrollBar(..) => true,
                                    _ => false
                                }) {
                                    if let UINodeKind::ScrollBar(scroll_bar) = scroll_bar_node.get_kind_mut() {
                                        scroll_bar.is_dragging = false;
                                    }
                                }
                                ui.release_mouse_capture();
                                evt.handled = true;
                            }))
                            .with_handler(RoutedEventHandlerType::MouseMove, Box::new(move |ui, handle, evt| {
                                let mouse_pos = match evt.kind {
                                    RoutedEventKind::MouseMove { pos } => pos,
                                    _ => return
                                };

                                let (field_pos, field_size) =
                                    match ui.borrow_by_name_up(&handle, ScrollBar::PART_CANVAS) {
                                        Some(canvas) => (canvas.screen_position, canvas.actual_size.get()),
                                        None => return
                                    };

                                let bar_size = match ui.nodes.borrow(&handle) {
                                    Some(node) => node.actual_size.get(),
                                    None => return
                                };

                                let new_value;

                                let scroll_bar_handle = ui.find_by_criteria_up(&handle, |node| match node.kind {
                                    UINodeKind::ScrollBar(..) => true,
                                    _ => false
                                });

                                if let Some(scroll_bar_node) = ui.nodes.borrow_mut(&scroll_bar_handle) {
                                    if let UINodeKind::ScrollBar(scroll_bar) = scroll_bar_node.get_kind_mut() {
                                        let orientation = scroll_bar.orientation;

                                        if scroll_bar.is_dragging {
                                            let percent = match orientation {
                                                Orientation::Horizontal => {
                                                    let span = field_size.x - bar_size.x;
                                                    let offset = mouse_pos.x - field_pos.x + scroll_bar.offset.x;
                                                    if span > 0.0 {
                                                        math::clampf(offset / span, 0.0, 1.0)
                                                    } else {
                                                        0.0
                                                    }
                                                }
                                                Orientation::Vertical => {
                                                    let span = field_size.y - bar_size.y;
                                                    let offset = mouse_pos.y - field_pos.y + scroll_bar.offset.y;
                                                    if span > 0.0 {
                                                        math::clampf(offset / span, 0.0, 1.0)
                                                    } else {
                                                        0.0
                                                    }
                                                }
                                            };

                                            new_value = percent * (scroll_bar.max - scroll_bar.min);

                                            evt.handled = true;
                                        } else {
                                            return;
                                        }
                                    } else {
                                        return;
                                    }
                                } else {
                                    return;
                                }

                                ScrollBar::set_value(&scroll_bar_handle, ui, new_value);
                            }))
                            .build(ui)
                        )
                        .build(ui)
                    )
                    .with_child(ButtonBuilder::new()
                        .with_width(match orientation {
                            Orientation::Horizontal => 30.0,
                            Orientation::Vertical => std::f32::NAN
                        })
                        .with_height(match orientation {
                            Orientation::Horizontal => std::f32::NAN,
                            Orientation::Vertical => 30.0
                        })
                        .on_column(match orientation {
                            Orientation::Horizontal => 2,
                            Orientation::Vertical => 0
                        })
                        .on_row(match orientation {
                            Orientation::Horizontal => 0,
                            Orientation::Vertical => 2
                        })
                        .with_click(Box::new(move |ui, handle| {
                            let scroll_bar_handle = ui.find_by_criteria_up(&handle, |node| match node.kind {
                                UINodeKind::ScrollBar(..) => true,
                                _ => false
                            });

                            let new_value = if let Some(scroll_bar_node) = ui.nodes.borrow_mut(&scroll_bar_handle) {
                                if let UINodeKind::ScrollBar(scroll_bar) = scroll_bar_node.get_kind_mut() {
                                    scroll_bar.value + scroll_bar.step
                                } else {
                                    return;
                                }
                            } else {
                                return;
                            };

                            ScrollBar::set_value(&scroll_bar_handle, ui, new_value);
                        }))
                        .with_text(match orientation {
                            Orientation::Horizontal => ">",
                            Orientation::Vertical => "v"
                        })
                        .build(ui)
                    )
                    .build(ui)
                )
                .build(ui)
            )
            .build(ui)
    }
}

pub struct ScrollContentPresenter {
    owner_handle: Handle<UINode>,
    scroll: Vec2,
    vertical_scroll_allowed: bool,
    horizontal_scroll_allowed: bool,
}

impl Layout for ScrollContentPresenter {
    fn measure_override(&self, ui: &UserInterface, available_size: &Vec2) -> Vec2 {
        let size_for_child = Vec2::make(
            if self.horizontal_scroll_allowed {
                std::f32::INFINITY
            } else {
                available_size.x
            },
            if self.vertical_scroll_allowed {
                std::f32::INFINITY
            } else {
                available_size.y
            },
        );

        let mut desired_size = Vec2::new();

        if let Some(node) = ui.nodes.borrow(&self.owner_handle) {
            for child_handle in node.children.iter() {
                ui.measure(child_handle, &size_for_child);

                if let Some(child) = ui.nodes.borrow(child_handle) {
                    let child_desired_size = child.desired_size.get();
                    if child_desired_size.x > desired_size.x {
                        desired_size.x = child_desired_size.x;
                    }
                    if child_desired_size.y > desired_size.y {
                        desired_size.y = child_desired_size.y;
                    }
                }
            }
        }

        desired_size
    }

    fn arrange_override(&self, ui: &UserInterface, final_size: &Vec2) -> Vec2 {
        let child_rect = Rect::new(
            -self.scroll.x,
            -self.scroll.y,
            final_size.x + self.scroll.x,
            final_size.y + self.scroll.y,
        );

        if let Some(node) = ui.nodes.borrow(&self.owner_handle) {
            for child_handle in node.children.iter() {
                ui.arrange(child_handle, &child_rect);
            }
        }

        *final_size
    }
}

impl ScrollContentPresenter {
    fn new() -> Self {
        Self {
            owner_handle: Handle::none(),
            scroll: Vec2::new(),
            vertical_scroll_allowed: true,
            horizontal_scroll_allowed: true,
        }
    }

    pub fn set_scroll(handle: &Handle<UINode>, ui: &mut UserInterface, scroll: Vec2) {
        if let Some(scp_node) = ui.nodes.borrow_mut(handle) {
            if let UINodeKind::ScrollContentPresenter(scp) = scp_node.get_kind_mut() {
                scp.scroll = scroll;
            }
        }
    }

    pub fn set_vertical_scroll(handle: &Handle<UINode>, ui: &mut UserInterface, scroll: f32) {
        if let Some(scp_node) = ui.nodes.borrow_mut(handle) {
            if let UINodeKind::ScrollContentPresenter(scp) = scp_node.get_kind_mut() {
                scp.scroll.y = scroll;
            }
        }
    }

    pub fn set_horizontal_scroll(handle: &Handle<UINode>, ui: &mut UserInterface, scroll: f32) {
        if let Some(scp_node) = ui.nodes.borrow_mut(handle) {
            if let UINodeKind::ScrollContentPresenter(scp) = scp_node.get_kind_mut() {
                scp.scroll.x = scroll;
            }
        }
    }
}

pub struct ScrollContentPresenterBuilder {
    vertical_scroll_allowed: Option<bool>,
    horizontal_scroll_allowed: Option<bool>,
    content: Option<Handle<UINode>>,
    common: CommonBuilderFields,
}

impl ScrollContentPresenterBuilder {
    pub fn new() -> Self {
        Self {
            vertical_scroll_allowed: None,
            horizontal_scroll_allowed: None,
            common: CommonBuilderFields::new(),
            content: None,
        }
    }

    impl_default_builder_methods!();

    pub fn with_content(mut self, content: Handle<UINode>) -> Self {
        self.content = Some(content);
        self
    }

    pub fn with_vertical_scroll_allowed(mut self, value: bool) -> Self {
        self.vertical_scroll_allowed = Some(value);
        self
    }

    pub fn with_horizontal_scroll_allowed(mut self, value: bool) -> Self {
        self.horizontal_scroll_allowed = Some(value);
        self
    }

    pub fn build(self, ui: &mut UserInterface) -> Handle<UINode> {
        let mut scp = ScrollContentPresenter::new();
        if let Some(vertical_scroll_allowed) = self.vertical_scroll_allowed {
            scp.vertical_scroll_allowed = vertical_scroll_allowed;
        }
        if let Some(horizontal_scroll_allowed) = self.horizontal_scroll_allowed {
            scp.horizontal_scroll_allowed = horizontal_scroll_allowed;
        }
        GenericNodeBuilder::new(UINodeKind::ScrollContentPresenter(scp), self.common)
            .with_child(self.content.unwrap_or(Handle::none()))
            .build(ui)
    }
}

pub struct ScrollViewer {
    owner_handle: Handle<UINode>,
    content_presenter: Handle<UINode>,
    v_scroll_bar: Handle<UINode>,
    h_scroll_bar: Handle<UINode>,
}

impl ScrollViewer {
    pub fn update(handle: &Handle<UINode>, ui: &mut UserInterface) {
        let mut content_size = Vec2::new();
        let mut available_size_for_content = Vec2::new();
        let mut horizontal_scroll_bar_handle = Handle::none();
        let mut vertical_scroll_bar_handle = Handle::none();

        if let Some(node) = ui.nodes.borrow(handle) {
            if let UINodeKind::ScrollViewer(scroll_viewer) = node.get_kind() {
                horizontal_scroll_bar_handle = scroll_viewer.h_scroll_bar.clone();
                vertical_scroll_bar_handle = scroll_viewer.v_scroll_bar.clone();
                if let Some(content_presenter) = ui.nodes.borrow(&scroll_viewer.content_presenter) {
                    available_size_for_content = content_presenter.desired_size.get();
                    for content_handle in content_presenter.children.iter() {
                        if let Some(content) = ui.nodes.borrow(content_handle) {
                            let content_desired_size = content.desired_size.get();
                            if content_desired_size.x > content_size.x {
                                content_size.x = content_desired_size.x;
                            }
                            if content_desired_size.y > content_size.y {
                                content_size.y = content_desired_size.y;
                            }
                        }
                    }
                }
            }
        }

        // Then adjust scroll bars according to content size.
        ScrollBar::set_max_value(&horizontal_scroll_bar_handle, ui, maxf(0.0, content_size.x - available_size_for_content.x));
        ScrollBar::set_max_value(&vertical_scroll_bar_handle, ui, maxf(0.0, content_size.y - available_size_for_content.y));
    }
}

pub struct ScrollViewerBuilder {
    common: CommonBuilderFields,
    content: Option<Handle<UINode>>,
}

impl ScrollViewerBuilder {
    pub fn new() -> Self {
        Self {
            common: CommonBuilderFields::new(),
            content: None,
        }
    }

    impl_default_builder_methods!();

    pub fn with_content(mut self, content: Handle<UINode>) -> Self {
        self.content = Some(content);
        self
    }

    pub fn build(self, ui: &mut UserInterface) -> Handle<UINode> {
        let content_presenter = ScrollContentPresenterBuilder::new()
            .with_child(ButtonBuilder::new()
                .with_text("TEST CONTENT")
                .with_width(300.0)
                .with_height(300.0)
                .build(ui))
            .on_row(0)
            .on_column(0)
            .build(ui);

        let v_scroll_bar = ScrollBarBuilder::new()
            .with_orientation(Orientation::Vertical)
            .on_row(0)
            .on_column(1)
            .with_value_changed({
                let content_presenter = content_presenter.clone();
                Box::new(move |ui, args| {
                    ScrollContentPresenter::set_vertical_scroll(&content_presenter, ui, args.new_value);
                })
            })
            .build(ui);

        let h_scroll_bar = ScrollBarBuilder::new()
            .with_orientation(Orientation::Horizontal)
            .on_row(1)
            .on_column(0)
            .with_value_changed({
                let content_presenter = content_presenter.clone();
                Box::new(move |ui, args| {
                    ScrollContentPresenter::set_horizontal_scroll(&content_presenter, ui, args.new_value);
                })
            })
            .build(ui);

        let mut scroll_viewer = ScrollViewer {
            owner_handle: Handle::none(),
            v_scroll_bar: v_scroll_bar.clone(),
            h_scroll_bar: h_scroll_bar.clone(),
            content_presenter: content_presenter.clone(),
        };

        GenericNodeBuilder::new(UINodeKind::ScrollViewer(scroll_viewer), self.common)
            .with_child(GridBuilder::new()
                .add_row(Row::stretch())
                .add_row(Row::strict(20.0))
                .add_column(Column::stretch())
                .add_column(Column::strict(20.0))
                .with_child(content_presenter)
                .with_child(h_scroll_bar)
                .with_child(v_scroll_bar)
                .build(ui))
            .build(ui)
    }
}

#[derive(PartialEq)]
pub enum SizeMode {
    Strict,
    Auto,
    Stretch,
}

pub struct Column {
    size_mode: SizeMode,
    desired_width: f32,
    actual_width: f32,
    x: f32,
}

impl Column {
    pub fn generic(size_mode: SizeMode, desired_width: f32) -> Self {
        Column {
            size_mode,
            desired_width,
            actual_width: 0.0,
            x: 0.0,
        }
    }

    pub fn strict(desired_width: f32) -> Self {
        Self {
            size_mode: SizeMode::Strict,
            desired_width,
            actual_width: 0.0,
            x: 0.0,
        }
    }

    pub fn stretch() -> Self {
        Self {
            size_mode: SizeMode::Stretch,
            desired_width: 0.0,
            actual_width: 0.0,
            x: 0.0,
        }
    }

    pub fn auto() -> Self {
        Self {
            size_mode: SizeMode::Auto,
            desired_width: 0.0,
            actual_width: 0.0,
            x: 0.0,
        }
    }
}

pub struct Row {
    size_mode: SizeMode,
    desired_height: f32,
    actual_height: f32,
    y: f32,
}

impl Row {
    pub fn generic(size_mode: SizeMode, desired_height: f32) -> Self {
        Self {
            size_mode,
            desired_height,
            actual_height: 0.0,
            y: 0.0,
        }
    }

    pub fn strict(desired_height: f32) -> Self {
        Self {
            size_mode: SizeMode::Strict,
            desired_height,
            actual_height: 0.0,
            y: 0.0,
        }
    }

    pub fn stretch() -> Self {
        Self {
            size_mode: SizeMode::Stretch,
            desired_height: 0.0,
            actual_height: 0.0,
            y: 0.0,
        }
    }

    pub fn auto() -> Self {
        Self {
            size_mode: SizeMode::Auto,
            desired_height: 0.0,
            actual_height: 0.0,
            y: 0.0,
        }
    }
}

pub struct Grid {
    owner_handle: Handle<UINode>,
    rows: RefCell<Vec<Row>>,
    columns: RefCell<Vec<Column>>,
}

impl Grid {
    fn new() -> Self {
        Self {
            owner_handle: Handle::none(),
            rows: RefCell::new(Vec::new()),
            columns: RefCell::new(Vec::new()),
        }
    }
}

impl Layout for Grid {
    fn measure_override(&self, ui: &UserInterface, available_size: &Vec2) -> Vec2 {
        // In case of no rows or columns, grid acts like default panel.
        if self.columns.borrow().is_empty() || self.rows.borrow().is_empty() {
            return ui.default_measure_override(&self.owner_handle, available_size);
        }

        let mut desired_size = Vec2::new();
        if let Some(node) = ui.nodes.borrow(&self.owner_handle) {
            // Step 1. Measure every children with relaxed constraints (size of grid).
            for child_handle in node.children.iter() {
                ui.measure(child_handle, available_size);
            }

            // Step 2. Calculate width of columns and heights of rows.
            let mut preset_width = 0.0;
            let mut preset_height = 0.0;

            // Step 2.1. Calculate size of strict-sized and auto-sized columns.
            for (i, col) in self.columns.borrow_mut().iter_mut().enumerate() {
                if col.size_mode == SizeMode::Strict {
                    col.actual_width = col.desired_width;
                    preset_width += col.actual_width;
                } else if col.size_mode == SizeMode::Auto {
                    col.actual_width = col.desired_width;
                    for child_handle in node.children.iter() {
                        if let Some(child) = ui.nodes.borrow(child_handle) {
                            if child.column == i && child.visibility == Visibility::Visible && child.desired_size.get().x > col.actual_width {
                                col.actual_width = child.desired_size.get().x;
                            }
                        }
                    }
                    preset_width += col.actual_width;
                }
            }

            // Step 2.2. Calculate size of strict-sized and auto-sized rows.
            for (i, row) in self.rows.borrow_mut().iter_mut().enumerate() {
                if row.size_mode == SizeMode::Strict {
                    row.actual_height = row.desired_height;
                    preset_height += row.actual_height;
                } else if row.size_mode == SizeMode::Auto {
                    row.actual_height = row.desired_height;
                    for child_handle in node.children.iter() {
                        if let Some(child) = ui.nodes.borrow(child_handle) {
                            if child.row == i && child.visibility == Visibility::Visible && child.desired_size.get().y > row.actual_height {
                                row.actual_height = child.desired_size.get().y;
                            }
                        }
                    }
                    preset_height += row.actual_height;
                }
            }

            // Step 2.3. Fit stretch-sized columns

            let mut rest_width = 0.0;
            if available_size.x.is_infinite() {
                for child_handle in node.children.iter() {
                    if let Some(child) = ui.nodes.borrow(child_handle) {
                        if let Some(column) = self.columns.borrow().get(child.column) {
                            if column.size_mode == SizeMode::Stretch {
                                rest_width += child.desired_size.get().x;
                            }
                        }
                    }
                }
            } else {
                rest_width = available_size.x - preset_width;
            }

            // count columns first
            let mut stretch_sized_columns = 0;
            for column in self.columns.borrow().iter() {
                if column.size_mode == SizeMode::Stretch {
                    stretch_sized_columns += 1;
                }
            }
            if stretch_sized_columns > 0 {
                let width_per_col = rest_width / stretch_sized_columns as f32;
                for column in self.columns.borrow_mut().iter_mut() {
                    if column.size_mode == SizeMode::Stretch {
                        column.actual_width = width_per_col;
                    }
                }
            }

            // Step 2.4. Fit stretch-sized rows.
            let mut stretch_sized_rows = 0;
            let mut rest_height = 0.0;
            if available_size.y.is_infinite() {
                for child_handle in node.children.iter() {
                    if let Some(child) = ui.nodes.borrow(child_handle) {
                        if let Some(row) = self.rows.borrow().get(child.row) {
                            if row.size_mode == SizeMode::Stretch {
                                rest_height += child.desired_size.get().y;
                            }
                        }
                    }
                }
            } else {
                rest_height = available_size.y - preset_height;
            }
            // count rows first
            for row in self.rows.borrow().iter() {
                if row.size_mode == SizeMode::Stretch {
                    stretch_sized_rows += 1;
                }
            }
            if stretch_sized_rows > 0 {
                let height_per_row = rest_height / stretch_sized_rows as f32;
                for row in self.rows.borrow_mut().iter_mut() {
                    if row.size_mode == SizeMode::Stretch {
                        row.actual_height = height_per_row;
                    }
                }
            }

            // Step 2.5. Calculate positions of each column.
            let mut y = 0.0;
            for row in self.rows.borrow_mut().iter_mut() {
                row.y = y;
                y += row.actual_height;
            }

            // Step 2.6. Calculate positions of each row.
            let mut x = 0.0;
            for column in self.columns.borrow_mut().iter_mut() {
                column.x = x;
                x += column.actual_width;
            }

            // Step 3. Re-measure children with new constraints.
            for child_handle in node.children.iter() {
                let size_for_child = {
                    if let Some(child) = ui.nodes.borrow(child_handle) {
                        Vec2 {
                            x: self.columns.borrow()[child.column].actual_width,
                            y: self.rows.borrow()[child.row].actual_height,
                        }
                    } else {
                        Vec2 {
                            x: match self.columns.borrow().first() {
                                Some(column) => column.actual_width,
                                None => 0.0
                            },
                            y: match self.rows.borrow().first() {
                                Some(row) => row.actual_height,
                                None => 0.0
                            },
                        }
                    }
                };
                ui.measure(child_handle, &size_for_child);
            }

            // Step 4. Calculate desired size of grid.
            for column in self.columns.borrow().iter() {
                desired_size.x += column.actual_width;
            }
            for row in self.rows.borrow().iter() {
                desired_size.y += row.actual_height;
            }
        }

        desired_size
    }

    fn arrange_override(&self, ui: &UserInterface, final_size: &Vec2) -> Vec2 {
        if let Some(node) = ui.nodes.borrow(&self.owner_handle) {
            if self.columns.borrow().is_empty() || self.rows.borrow().is_empty() {
                let rect = Rect::new(0.0, 0.0, final_size.x, final_size.y);
                for child_handle in node.children.iter() {
                    ui.arrange(child_handle, &rect);
                }
                return *final_size;
            }

            for child_handle in node.children.iter() {
                let mut final_rect = None;

                if let Some(child) = ui.nodes.borrow(&child_handle) {
                    if let Some(column) = self.columns.borrow().get(child.column) {
                        if let Some(row) = self.rows.borrow().get(child.row) {
                            final_rect = Some(Rect::new(
                                column.x,
                                row.y,
                                column.actual_width,
                                row.actual_height,
                            ));
                        }
                    }
                }

                if let Some(rect) = final_rect {
                    ui.arrange(child_handle, &rect);
                }
            }
        }

        *final_size
    }
}

pub struct GridBuilder {
    rows: Vec<Row>,
    columns: Vec<Column>,
    common: CommonBuilderFields,
}

impl GridBuilder {
    pub fn new() -> Self {
        GridBuilder {
            rows: Vec::new(),
            columns: Vec::new(),
            common: CommonBuilderFields::new(),
        }
    }

    impl_default_builder_methods!();

    pub fn add_row(mut self, row: Row) -> Self {
        self.rows.push(row);
        self
    }

    pub fn add_column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    pub fn add_rows(mut self, mut rows: Vec<Row>) -> Self {
        self.rows.append(&mut rows);
        self
    }

    pub fn add_columns(mut self, mut columns: Vec<Column>) -> Self {
        self.columns.append(&mut columns);
        self
    }

    pub fn build(mut self, ui: &mut UserInterface) -> Handle<UINode> {
        let mut grid = Grid::new();
        grid.columns = RefCell::new(self.columns);
        grid.rows = RefCell::new(self.rows);

        let node = UINode::new(UINodeKind::Grid(grid));

        let handle = ui.add_node(node);
        self.common.apply(ui, &handle);
        handle
    }
}

impl Grid {
    pub fn add_row(&mut self, row: Row) -> &mut Self {
        self.rows.borrow_mut().push(row);
        self
    }

    pub fn add_column(&mut self, column: Column) -> &mut Self {
        self.columns.borrow_mut().push(column);
        self
    }
}

pub struct Canvas {
    owner_handle: Handle<UINode>
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            owner_handle: Handle::none()
        }
    }
}

impl Layout for Canvas {
    fn measure_override(&self, ui: &UserInterface, _available_size: &Vec2) -> Vec2 {
        let size_for_child = Vec2::make(
            std::f32::INFINITY,
            std::f32::INFINITY,
        );

        if let Some(node) = ui.nodes.borrow(&self.owner_handle) {
            for child_handle in node.children.iter() {
                ui.measure(child_handle, &size_for_child);
            }
        }

        Vec2::new()
    }

    fn arrange_override(&self, ui: &UserInterface, final_size: &Vec2) -> Vec2 {
        if let Some(node) = ui.nodes.borrow(&self.owner_handle) {
            for child_handle in node.children.iter() {
                let mut final_rect = None;

                if let Some(child) = ui.nodes.borrow(&child_handle) {
                    final_rect = Some(Rect::new(
                        child.desired_local_position.get().x,
                        child.desired_local_position.get().y,
                        child.desired_size.get().x,
                        child.desired_size.get().y));
                }

                if let Some(rect) = final_rect {
                    ui.arrange(child_handle, &rect);
                }
            }
        }

        *final_size
    }
}

pub enum UINodeKind {
    Text(Text),
    Border(Border),
    Button(Button),
    ScrollBar(ScrollBar),
    ScrollViewer(ScrollViewer),
    Image(Image),
    /// Automatically arranges children by rows and columns
    Grid(Grid),
    /// Allows user to directly set position and size of a node
    Canvas(Canvas),
    /// Allows user to scroll content
    ScrollContentPresenter(ScrollContentPresenter),
}

impl Drawable for UINodeKind {
    fn draw(&mut self, drawing_context: &mut DrawingContext, font_cache: &Pool<Font>, bounds: &Rect<f32>, color: Color) {
        match self {
            UINodeKind::Text(text) => text.draw(drawing_context, font_cache, bounds, color),
            UINodeKind::Border(border) => border.draw(drawing_context, font_cache, bounds, color),
            _ => ()
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum RoutedEventHandlerType {
    MouseMove,
    MouseEnter,
    MouseLeave,
    MouseDown,
    MouseUp,
    Count,
}

pub type RoutedEventHandler = dyn FnMut(&mut UserInterface, Handle<UINode>, &mut RoutedEvent);

pub type RoutedEventHandlerList = [Option<Box<RoutedEventHandler>>; RoutedEventHandlerType::Count as usize];

/// Notes. Some fields wrapped into Cell's to be able to modify them while in measure/arrange
/// stage. This is required evil, I can't just unwrap all the recursive calls in measure/arrange.
pub struct UINode {
    name: String,
    kind: UINodeKind,
    /// Desired position relative to parent node
    desired_local_position: Cell<Vec2>,
    /// Explicit width for node or automatic if NaN (means value is undefined). Default is NaN
    width: Cell<f32>,
    /// Explicit height for node or automatic if NaN (means value is undefined). Default is NaN
    height: Cell<f32>,
    /// Screen position of the node
    screen_position: Vec2,
    /// Desired size of the node after Measure pass.
    desired_size: Cell<Vec2>,
    /// Actual node local position after Arrange pass.
    actual_local_position: Cell<Vec2>,
    /// Actual size of the node after Arrange pass.
    actual_size: Cell<Vec2>,
    /// Minimum width and height
    min_size: Vec2,
    /// Maximum width and height
    max_size: Vec2,
    /// Overlay color of the node
    color: Color,
    /// Index of row to which this node belongs
    row: usize,
    /// Index of column to which this node belongs
    column: usize,
    /// Vertical alignment
    vertical_alignment: VerticalAlignment,
    /// Horizontal alignment
    horizontal_alignment: HorizontalAlignment,
    /// Margin (four sides)
    margin: Thickness,
    /// Current visibility state
    visibility: Visibility,
    children: Vec<Handle<UINode>>,
    parent: Handle<UINode>,
    /// Indices of commands in command buffer emitted by the node.
    command_indices: Vec<usize>,
    is_mouse_over: bool,
    event_handlers: RoutedEventHandlerList,
}

pub enum RoutedEventKind {
    MouseDown {
        pos: Vec2,
        button: MouseButton,
    },
    MouseUp {
        pos: Vec2,
        button: MouseButton,
    },
    MouseMove {
        pos: Vec2
    },
    Text {
        symbol: char
    },
    KeyDown {
        code: VirtualKeyCode
    },
    KeyUp {
        code: VirtualKeyCode
    },
    MouseWheel {
        pos: Vec2,
        amount: u32,
    },
    MouseLeave,
    MouseEnter,
}

pub struct RoutedEvent {
    kind: RoutedEventKind,
    handled: bool,
}

impl RoutedEvent {
    pub fn new(kind: RoutedEventKind) -> RoutedEvent {
        RoutedEvent {
            kind,
            handled: false,
        }
    }
}

pub type DeferredAction = dyn FnMut(&mut UserInterface);

pub struct UserInterface {
    nodes: Pool<UINode>,
    drawing_context: DrawingContext,
    default_font: Handle<Font>,
    visual_debug: bool,
    /// Every UI node will live on the window-sized canvas.
    root_canvas: Handle<UINode>,
    picked_node: Handle<UINode>,
    prev_picked_node: Handle<UINode>,
    captured_node: Handle<UINode>,
    mouse_position: Vec2,
    deferred_actions: VecDeque<Box<DeferredAction>>,
}

#[inline]
fn maxf(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

#[inline]
fn minf(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

struct ArrangeData {
    size: Vec2,
    size_without_margin: Vec2,
    origin: Vec2,
}

impl UserInterface {
    pub fn new(default_font: Handle<Font>) -> UserInterface {
        let mut ui = UserInterface {
            visual_debug: false,
            default_font,
            captured_node: Handle::none(),
            root_canvas: Handle::none(),
            nodes: Pool::new(),
            mouse_position: Vec2::new(),
            drawing_context: DrawingContext::new(),
            picked_node: Handle::none(),
            prev_picked_node: Handle::none(),
            deferred_actions: VecDeque::new(),
        };
        ui.root_canvas = ui.add_node(UINode::new(UINodeKind::Canvas(Canvas::new())));
        ui
    }

    pub fn add_node(&mut self, node: UINode) -> Handle<UINode> {
        let node_handle = self.nodes.spawn(node);
        // Notify kind about owner. This is a bit hackish but it'll make a lot of things easier.
        if let Some(node) = self.nodes.borrow_mut(&node_handle) {
            match &mut node.kind {
                UINodeKind::ScrollBar(scroll_bar) => scroll_bar.owner_handle = node_handle.clone(),
                UINodeKind::Text(text) => text.owner_handle = node_handle.clone(),
                UINodeKind::Border(border) => border.owner_handle = node_handle.clone(),
                UINodeKind::Button(button) => button.owner_handle = node_handle.clone(),
                UINodeKind::ScrollViewer(scroll_viewer) => scroll_viewer.owner_handle = node_handle.clone(),
                UINodeKind::Image(image) => image.owner_handle = node_handle.clone(),
                UINodeKind::Grid(grid) => grid.owner_handle = node_handle.clone(),
                UINodeKind::Canvas(canvas) => canvas.owner_handle = node_handle.clone(),
                UINodeKind::ScrollContentPresenter(scp) => scp.owner_handle = node_handle.clone(),
            }
        }
        self.link_nodes(&node_handle, &self.root_canvas.clone());
        node_handle
    }

    pub fn capture_mouse(&mut self, node: &Handle<UINode>) -> bool {
        if self.captured_node.is_none() && self.nodes.is_valid_handle(node) {
            self.captured_node = node.clone();
            return true;
        }

        false
    }

    pub fn release_mouse_capture(&mut self) {
        self.captured_node = Handle::none();
    }

    pub fn begin_invoke(&mut self, action: Box<DeferredAction>) {
        self.deferred_actions.push_back(action)
    }

    /// Links specified child with specified parent.
    #[inline]
    pub fn link_nodes(&mut self, child_handle: &Handle<UINode>, parent_handle: &Handle<UINode>) {
        self.unlink_node(child_handle);
        if let Some(child) = self.nodes.borrow_mut(child_handle) {
            child.parent = parent_handle.clone();
            if let Some(parent) = self.nodes.borrow_mut(parent_handle) {
                parent.children.push(child_handle.clone());
            }
        }
    }

    /// Unlinks specified node from its parent, so node will become root.
    #[inline]
    pub fn unlink_node(&mut self, node_handle: &Handle<UINode>) {
        let mut parent_handle: Handle<UINode> = Handle::none();
        // Replace parent handle of child
        if let Some(node) = self.nodes.borrow_mut(node_handle) {
            parent_handle = node.parent.clone();
            node.parent = Handle::none();
        }
        // Remove child from parent's children list
        if let Some(parent) = self.nodes.borrow_mut(&parent_handle) {
            if let Some(i) = parent.children.iter().position(|h| h == node_handle) {
                parent.children.remove(i);
            }
        }
    }

    #[inline]
    pub fn get_node(&self, node_handle: &Handle<UINode>) -> Option<&UINode> {
        self.nodes.borrow(node_handle)
    }

    #[inline]
    pub fn get_node_mut(&mut self, node_handle: &Handle<UINode>) -> Option<&mut UINode> {
        self.nodes.borrow_mut(node_handle)
    }

    #[inline]
    pub fn get_drawing_context(&self) -> &DrawingContext {
        &self.drawing_context
    }

    #[inline]
    pub fn get_drawing_context_mut(&mut self) -> &mut DrawingContext {
        &mut self.drawing_context
    }

    fn default_measure_override(&self, handle: &Handle<UINode>, available_size: &Vec2) -> Vec2 {
        let mut size = Vec2::new();

        if let Some(node) = self.nodes.borrow(handle) {
            for child_handle in node.children.iter() {
                self.measure(child_handle, &available_size);

                if let Some(child) = self.nodes.borrow(child_handle) {
                    let child_desired_size = child.desired_size.get();
                    if child_desired_size.x > size.x {
                        size.x = child_desired_size.x;
                    }
                    if child_desired_size.y > size.y {
                        size.y = child_desired_size.y;
                    }
                }
            }
        }

        size
    }

    fn measure(&self, node_handle: &Handle<UINode>, available_size: &Vec2) {
        match self.nodes.borrow(&node_handle) {
            None => return,
            Some(node) => {
                let margin = Vec2 {
                    x: node.margin.left + node.margin.right,
                    y: node.margin.top + node.margin.bottom,
                };

                let size_for_child = Vec2 {
                    x: {
                        let w = if node.width.get() > 0.0 {
                            node.width.get()
                        } else {
                            maxf(0.0, available_size.x - margin.x)
                        };

                        if w > node.max_size.x {
                            node.max_size.x
                        } else if w < node.min_size.x {
                            node.min_size.x
                        } else {
                            w
                        }
                    },
                    y: {
                        let h = if node.height.get() > 0.0 {
                            node.height.get()
                        } else {
                            maxf(0.0, available_size.y - margin.y)
                        };

                        if h > node.max_size.y {
                            node.max_size.y
                        } else if h < node.min_size.y {
                            node.min_size.y
                        } else {
                            h
                        }
                    },
                };

                if node.visibility == Visibility::Visible {
                    let mut desired_size = match &node.kind {
                        UINodeKind::Border(border) => border.measure_override(self, &size_for_child),
                        UINodeKind::Canvas(canvas) => canvas.measure_override(self, &size_for_child),
                        UINodeKind::Grid(grid) => grid.measure_override(self, &size_for_child),
                        UINodeKind::ScrollContentPresenter(scp) => scp.measure_override(self, &size_for_child),
                        UINodeKind::ScrollBar(scroll_bar) => scroll_bar.measure_override(self, &size_for_child),
                        _ => self.default_measure_override(node_handle, &size_for_child)
                    };

                    if !node.width.get().is_nan() {
                        desired_size.x = node.width.get();
                    }

                    if desired_size.x > node.max_size.x {
                        desired_size.x = node.max_size.x;
                    } else if desired_size.x < node.min_size.x {
                        desired_size.x = node.min_size.x;
                    }

                    if desired_size.y > node.max_size.y {
                        desired_size.y = node.max_size.y;
                    } else if desired_size.y < node.min_size.y {
                        desired_size.y = node.min_size.y;
                    }

                    if !node.height.get().is_nan() {
                        desired_size.y = node.height.get();
                    }

                    desired_size += margin;

                    // Make sure that node won't go outside of available bounds.
                    if desired_size.x > available_size.x {
                        desired_size.x = available_size.x;
                    }
                    if desired_size.y > available_size.y {
                        desired_size.y = available_size.y;
                    }

                    node.desired_size.set(desired_size);
                } else {
                    node.desired_size.set(Vec2::make(0.0, 0.0));
                }
            }
        }
    }

    fn default_arrange_override(&self, handle: &Handle<UINode>, final_size: &Vec2) -> Vec2 {
        let final_rect = Rect::new(0.0, 0.0, final_size.x, final_size.y);

        if let Some(node) = self.nodes.borrow(handle) {
            for child_handle in node.children.iter() {
                self.arrange(child_handle, &final_rect);
            }
        }

        *final_size
    }

    fn arrange(&self, node_handle: &Handle<UINode>, final_rect: &Rect<f32>) {
        match self.nodes.borrow(node_handle) {
            None => return,
            Some(node) => {
                if node.visibility != Visibility::Visible {
                    return;
                }

                let margin_x = node.margin.left + node.margin.right;
                let margin_y = node.margin.top + node.margin.bottom;

                let mut origin_x = final_rect.x + node.margin.left;
                let mut origin_y = final_rect.y + node.margin.top;

                let mut size = Vec2 {
                    x: maxf(0.0, final_rect.w - margin_x),
                    y: maxf(0.0, final_rect.h - margin_y),
                };

                let size_without_margin = size;

                if node.horizontal_alignment != HorizontalAlignment::Stretch {
                    size.x = minf(size.x, node.desired_size.get().x - margin_x);
                }
                if node.vertical_alignment != VerticalAlignment::Stretch {
                    size.y = minf(size.y, node.desired_size.get().y - margin_y);
                }

                if node.width.get() > 0.0 {
                    size.x = node.width.get();
                }
                if node.height.get() > 0.0 {
                    size.y = node.height.get();
                }

                size = match &node.kind {
                    UINodeKind::Border(border) => border.arrange_override(self, &size),
                    UINodeKind::Canvas(canvas) => canvas.arrange_override(self, &size),
                    UINodeKind::Grid(grid) => grid.arrange_override(self, &size),
                    UINodeKind::ScrollContentPresenter(scp) => scp.arrange_override(self, &size),
                    UINodeKind::ScrollBar(scroll_bar) => scroll_bar.arrange_override(self, &size),
                    _ => self.default_arrange_override(node_handle, &size)
                };

                if size.x > final_rect.w {
                    size.x = final_rect.w;
                }
                if size.y > final_rect.h {
                    size.y = final_rect.h;
                }

                match node.horizontal_alignment {
                    HorizontalAlignment::Center | HorizontalAlignment::Stretch => {
                        origin_x += (size_without_margin.x - size.x) * 0.5;
                    }
                    HorizontalAlignment::Right => {
                        origin_x += size_without_margin.x - size.x;
                    }
                    _ => ()
                }

                match node.vertical_alignment {
                    VerticalAlignment::Center | VerticalAlignment::Stretch => {
                        origin_y += (size_without_margin.y - size.y) * 0.5;
                    }
                    VerticalAlignment::Bottom => {
                        origin_y += size_without_margin.y - size.y;
                    }
                    _ => ()
                }

                node.actual_size.set(size);
                node.actual_local_position.set(Vec2 { x: origin_x, y: origin_y });
            }
        }
    }

    fn update_transform(&mut self, node_handle: &Handle<UINode>) {
        let mut children = UnsafeCollectionView::empty();

        let mut screen_position = Vec2::new();
        if let Some(node) = self.nodes.borrow(node_handle) {
            children = UnsafeCollectionView::from_vec(&node.children);
            if let Some(parent) = self.nodes.borrow(&node.parent) {
                screen_position = node.actual_local_position.get() + parent.screen_position;
            } else {
                screen_position = node.actual_local_position.get();
            }
        }

        if let Some(node) = self.nodes.borrow_mut(node_handle) {
            node.screen_position = screen_position;
        }

        // Continue on children
        for child_handle in children.iter() {
            self.update_transform(child_handle);
        }
    }


    pub fn update(&mut self, screen_size: &Vec2) {
        let root_canvas_handle = self.root_canvas.clone();
        self.measure(&root_canvas_handle, screen_size);
        self.arrange(&root_canvas_handle, &Rect::new(0.0, 0.0, screen_size.x, screen_size.y));
        self.update_transform(&root_canvas_handle);

        // Do deferred actions. Some sort of simplest dispatcher.
        while let Some(mut action) = self.deferred_actions.pop_front() {
            action(self)
        }

        for i in 0..self.nodes.get_capacity() {
            let id = if let Some(node) = self.nodes.at(i) {
                node.get_kind_id()
            } else {
                continue;
            };

            let handle = self.nodes.handle_from_index(i);
            if id == TypeId::of::<ScrollViewer>() {
                ScrollViewer::update(&handle, self);
            }
        }
    }

    fn draw_node(&mut self, node_handle: &Handle<UINode>, font_cache: &Pool<Font>, nesting: u8) {
        let mut children: UnsafeCollectionView<Handle<UINode>> = UnsafeCollectionView::empty();

        if let Some(node) = self.nodes.borrow_mut(node_handle) {
            let start_index = self.drawing_context.get_commands().len();
            let bounds = node.get_screen_bounds();

            self.drawing_context.set_nesting(nesting);
            self.drawing_context.commit_clip_rect(&bounds.inflate(0.9, 0.9));

            node.kind.draw(&mut self.drawing_context, font_cache, &bounds, node.color);

            children = UnsafeCollectionView::from_vec(&node.children);

            let end_index = self.drawing_context.get_commands().len();
            for i in start_index..end_index {
                node.command_indices.push(i);
            }
        }

        // Continue on children
        for child_node in children.iter() {
            self.draw_node(child_node, font_cache, nesting + 1);
        }

        self.drawing_context.revert_clip_geom();
    }

    pub fn draw(&mut self, font_cache: &Pool<Font>) -> &DrawingContext {
        self.drawing_context.clear();

        let root_canvas = self.root_canvas.clone();
        self.draw_node(&root_canvas, font_cache, 1);

        if self.visual_debug {
            self.drawing_context.set_nesting(0);

            let picked_bounds =
                if let Some(picked_node) = self.nodes.borrow(&self.picked_node) {
                    Some(picked_node.get_screen_bounds())
                } else {
                    None
                };

            if let Some(picked_bounds) = picked_bounds {
                self.drawing_context.push_rect(&picked_bounds, 1.0, Color::white());
                self.drawing_context.commit(CommandKind::Geometry, 0);
            }
        }

        &self.drawing_context
    }

    fn is_node_clipped(&self, node_handle: &Handle<UINode>, pt: &Vec2) -> bool {
        let mut clipped = true;

        if let Some(node) = self.nodes.borrow(node_handle) {
            if node.visibility != Visibility::Visible {
                return clipped;
            }

            for command_index in node.command_indices.iter() {
                if let Some(command) = self.drawing_context.get_commands().get(*command_index) {
                    if *command.get_kind() == CommandKind::Clip && self.drawing_context.is_command_contains_point(command, pt) {
                        clipped = false;
                        break;
                    }
                }
            }

            // Point can be clipped by parent's clipping geometry.
            if !node.parent.is_none() && !clipped {
                clipped |= self.is_node_clipped(&node.parent, pt);
            }
        }

        clipped
    }

    fn is_node_contains_point(&self, node_handle: &Handle<UINode>, pt: &Vec2) -> bool {
        if let Some(node) = self.nodes.borrow(node_handle) {
            if node.visibility != Visibility::Visible {
                return false;
            }

            if !self.is_node_clipped(node_handle, pt) {
                for command_index in node.command_indices.iter() {
                    if let Some(command) = self.drawing_context.get_commands().get(*command_index) {
                        if *command.get_kind() == CommandKind::Geometry && self.drawing_context.is_command_contains_point(command, pt) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    fn pick_node(&self, node_handle: &Handle<UINode>, pt: &Vec2, level: &mut i32) -> Handle<UINode> {
        let mut picked = Handle::none();
        let mut topmost_picked_level = 0;

        if self.is_node_contains_point(node_handle, pt) {
            picked = node_handle.clone();
            topmost_picked_level = *level;
        }

        if let Some(node) = self.nodes.borrow(node_handle) {
            for child_handle in node.children.iter() {
                *level += 1;
                let picked_child = self.pick_node(child_handle, pt, level);
                if !picked_child.is_none() && *level > topmost_picked_level {
                    topmost_picked_level = *level;
                    picked = picked_child;
                }
            }
        }

        picked
    }

    pub fn hit_test(&self, pt: &Vec2) -> Handle<UINode> {
        if self.nodes.is_valid_handle(&self.captured_node) {
            self.captured_node.clone()
        } else {
            let mut level = 0;
            self.pick_node(&self.root_canvas, pt, &mut level)
        }
    }

    fn route_event(&mut self, node_handle: Handle<UINode>, event_type: RoutedEventHandlerType, event_args: &mut RoutedEvent) {
        let mut handler = None;
        let mut parent = Handle::none();
        let index = event_type as usize;

        if let Some(node) = self.nodes.borrow_mut(&node_handle) {
            // Take event handler.
            handler = node.event_handlers[index].take();
            parent = node.parent.clone();
        }

        // Execute event handler.
        if let Some(ref mut mouse_enter) = handler {
            mouse_enter(self, node_handle.clone(), event_args);
        }

        if let Some(node) = self.nodes.borrow_mut(&node_handle) {
            // Put event handler back.
            node.event_handlers[index] = handler.take();
        }

        // Route event up on hierarchy (bubbling strategy) until is not handled.
        if !event_args.handled && !parent.is_none() {
            self.route_event(parent, event_type, event_args);
        }
    }

    /// Searches a node down on tree starting from give root that matches a criteria
    /// defined by a given func.
    pub fn find_by_criteria_down<Func>(&self, node_handle: &Handle<UINode>, func: &Func) -> Handle<UINode>
        where Func: Fn(&UINode) -> bool {
        if let Some(node) = self.nodes.borrow(node_handle) {
            if func(node) {
                return node_handle.clone();
            }

            for child_handle in node.children.iter() {
                let result = self.find_by_criteria_down(child_handle, func);

                if result.is_some() {
                    return result;
                }
            }
        }
        Handle::none()
    }

    /// Searches a node up on tree starting from given root that matches a criteria
    /// defined by a given func.
    pub fn find_by_criteria_up<Func>(&self, node_handle: &Handle<UINode>, func: Func) -> Handle<UINode>
        where Func: Fn(&UINode) -> bool {
        if let Some(node) = self.nodes.borrow(node_handle) {
            if func(node) {
                return node_handle.clone();
            }

            return self.find_by_criteria_up(&node.parent, func);
        }

        Handle::none()
    }

    /// Searches a node by name up on tree starting from given root node.
    pub fn find_by_name_up(&self, node_handle: &Handle<UINode>, name: &str) -> Handle<UINode> {
        self.find_by_criteria_up(node_handle, |node| node.name == name)
    }

    /// Searches a node by name down on tree starting from given root node.
    pub fn find_by_name_down(&self, node_handle: &Handle<UINode>, name: &str) -> Handle<UINode> {
        self.find_by_criteria_down(node_handle, &|node| node.name == name)
    }

    /// Searches a node by name up on tree starting from given root node and tries to borrow it if exists.
    pub fn borrow_by_name_up(&self, start_node_handle: &Handle<UINode>, name: &str) -> Option<&UINode> {
        self.nodes.borrow(&self.find_by_name_up(start_node_handle, name))
    }

    /// Searches a node by name up on tree starting from given root node and tries to borrow it as mutable if exists.
    pub fn borrow_by_name_up_mut(&mut self, start_node_handle: &Handle<UINode>, name: &str) -> Option<&mut UINode> {
        self.nodes.borrow_mut(&self.find_by_name_up(start_node_handle, name))
    }

    /// Searches a node by name down on tree starting from given root node and tries to borrow it if exists.
    pub fn borrow_by_name_down(&self, start_node_handle: &Handle<UINode>, name: &str) -> Option<&UINode> {
        self.nodes.borrow(&self.find_by_name_down(start_node_handle, name))
    }

    /// Searches a node by name down on tree starting from given root node and tries to borrow it as mutable if exists.
    pub fn borrow_by_name_down_mut(&mut self, start_node_handle: &Handle<UINode>, name: &str) -> Option<&mut UINode> {
        self.nodes.borrow_mut(&self.find_by_name_down(start_node_handle, name))
    }

    pub fn borrow_by_criteria_up<Func>(&self, start_node_handle: &Handle<UINode>, func: Func) -> Option<&UINode>
        where Func: Fn(&UINode) -> bool {
        self.nodes.borrow(&self.find_by_criteria_up(start_node_handle, func))
    }

    pub fn borrow_by_criteria_up_mut<Func>(&mut self, start_node_handle: &Handle<UINode>, func: Func) -> Option<&mut UINode>
        where Func: Fn(&UINode) -> bool {
        self.nodes.borrow_mut(&self.find_by_criteria_up(start_node_handle, func))
    }

    pub fn get_node_kind_id(&self, handle: &Handle<UINode>) -> TypeId {
        if let Some(node) = self.nodes.borrow(handle) {
            node.get_kind_id()
        } else {
            TypeId::of::<()>()
        }
    }

    pub fn process_event(&mut self, event: &glutin::WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = Vec2::make(position.x as f32, position.y as f32);
                self.picked_node = self.hit_test(&self.mouse_position);

                // Fire mouse leave for previously picked node
                if self.picked_node != self.prev_picked_node {
                    let mut fire_mouse_leave = false;
                    if let Some(prev_picked_node) = self.nodes.borrow_mut(&self.prev_picked_node) {
                        if prev_picked_node.is_mouse_over {
                            prev_picked_node.is_mouse_over = false;
                            fire_mouse_leave = true;
                        }
                    }

                    if fire_mouse_leave {
                        let mut evt = RoutedEvent::new(RoutedEventKind::MouseLeave);
                        self.route_event(self.prev_picked_node.clone(), RoutedEventHandlerType::MouseLeave, &mut evt);
                    }
                }

                if !self.picked_node.is_none() {
                    let mut fire_mouse_enter = false;
                    if let Some(picked_node) = self.nodes.borrow_mut(&self.picked_node) {
                        if !picked_node.is_mouse_over {
                            picked_node.is_mouse_over = true;
                            fire_mouse_enter = true;
                        }
                    }

                    if fire_mouse_enter {
                        let mut evt = RoutedEvent::new(RoutedEventKind::MouseEnter);
                        self.route_event(self.picked_node.clone(), RoutedEventHandlerType::MouseEnter, &mut evt);
                    }

                    // Fire mouse move
                    let mut evt = RoutedEvent::new(RoutedEventKind::MouseMove {
                        pos: self.mouse_position
                    });
                    self.route_event(self.picked_node.clone(), RoutedEventHandlerType::MouseMove, &mut evt);
                }
            }
            _ => ()
        }

        if !self.picked_node.is_none() {
            match event {
                WindowEvent::MouseInput { button, state, .. } => {
                    match state {
                        ElementState::Pressed => {
                            let mut evt = RoutedEvent::new(RoutedEventKind::MouseDown {
                                pos: self.mouse_position,
                                button: *button,
                            });
                            self.route_event(self.picked_node.clone(), RoutedEventHandlerType::MouseDown, &mut evt);
                        }
                        ElementState::Released => {
                            let mut evt = RoutedEvent::new(RoutedEventKind::MouseUp {
                                pos: self.mouse_position,
                                button: *button,
                            });
                            self.route_event(self.picked_node.clone(), RoutedEventHandlerType::MouseUp, &mut evt);
                        }
                    }
                }
                _ => ()
            }
        }

        self.prev_picked_node = self.picked_node.clone();

        false
    }
}

impl UINode {
    pub fn new(kind: UINodeKind) -> UINode {
        UINode {
            kind,
            name: String::new(),
            desired_local_position: Cell::new(Vec2::new()),
            width: Cell::new(std::f32::NAN),
            height: Cell::new(std::f32::NAN),
            screen_position: Vec2::new(),
            desired_size: Cell::new(Vec2::new()),
            actual_local_position: Cell::new(Vec2::new()),
            actual_size: Cell::new(Vec2::new()),
            min_size: Vec2::make(0.0, 0.0),
            max_size: Vec2::make(std::f32::INFINITY, std::f32::INFINITY),
            color: Color::white(),
            row: 0,
            column: 0,
            vertical_alignment: VerticalAlignment::Stretch,
            horizontal_alignment: HorizontalAlignment::Stretch,
            margin: Thickness::zero(),
            visibility: Visibility::Visible,
            children: Vec::new(),
            parent: Handle::none(),
            command_indices: Vec::new(),
            event_handlers: Default::default(),
            is_mouse_over: false,
        }
    }

    #[inline]
    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    #[inline]
    pub fn set_width(&mut self, width: f32) -> &mut Self {
        self.width.set(width);
        self
    }

    #[inline]
    pub fn set_height(&mut self, height: f32) -> &mut Self {
        self.height.set(height);
        self
    }

    #[inline]
    pub fn set_desired_local_position(&self, pos: Vec2) -> &Self {
        self.desired_local_position.set(pos);
        self
    }

    #[inline]
    pub fn set_desired_local_position_mut(&mut self, pos: Vec2) -> &mut Self {
        self.desired_local_position.set(pos);
        self
    }

    #[inline]
    pub fn get_kind(&self) -> &UINodeKind {
        &self.kind
    }

    #[inline]
    pub fn set_vertical_alignment(&mut self, valign: VerticalAlignment) -> &mut Self {
        self.vertical_alignment = valign;
        self
    }

    #[inline]
    pub fn set_horizontal_alignment(&mut self, halign: HorizontalAlignment) -> &mut Self {
        self.horizontal_alignment = halign;
        self
    }

    #[inline]
    pub fn get_kind_mut(&mut self) -> &mut UINodeKind {
        &mut self.kind
    }

    #[inline]
    pub fn get_screen_bounds(&self) -> Rect<f32> {
        Rect::new(self.screen_position.x, self.screen_position.y, self.actual_size.get().x, self.actual_size.get().y)
    }

    #[inline]
    pub fn set_handler(&mut self, handler_type: RoutedEventHandlerType, handler: Box<RoutedEventHandler>) -> &mut Self {
        self.event_handlers[handler_type as usize] = Some(handler);
        self
    }

    pub fn get_kind_id(&self) -> TypeId {
        match &self.kind {
            UINodeKind::ScrollBar(scroll_bar) => scroll_bar.type_id(),
            UINodeKind::Text(text) => text.type_id(),
            UINodeKind::Border(border) => border.type_id(),
            UINodeKind::Button(button) => button.type_id(),
            UINodeKind::ScrollViewer(scroll_viewer) => scroll_viewer.type_id(),
            UINodeKind::Image(image) => image.type_id(),
            UINodeKind::Grid(grid) => grid.type_id(),
            UINodeKind::Canvas(canvas) => canvas.type_id(),
            UINodeKind::ScrollContentPresenter(scp) => scp.type_id()
        }
    }
}