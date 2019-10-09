use rg3d_core::{
    pool::{Handle, Pool},
    math::mat4::Mat4,
    visitor::{Visit, Visitor, VisitResult},
};
use crate::scene::{
    node::{Node, NodeKind},
    SceneInterface,
};
use std::collections::HashMap;
use rg3d_core::pool::{PoolIterator, PoolIteratorMut, PoolPairIterator};

pub struct Graph {
    root: Handle<Node>,
    pool: Pool<Node>,
    stack: Vec<Handle<Node>>,
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            root: Handle::NONE,
            pool: Pool::new(),
            stack: Vec::new(),
        }
    }
}

impl Graph {
    /// Creates new graph instance with single root node.
    pub fn new() -> Self {
        let mut pool: Pool<Node> = Pool::new();
        let mut root = Node::new(NodeKind::Base);
        root.set_name("__ROOT__");
        let root = pool.spawn(root);
        Self {
            stack: Vec::new(),
            root,
            pool,
        }
    }

    /// Adds new node to the graph. Node will be transferred into implementation-defined
    /// storage and you'll get a handle to the node. Node will be automatically attached
    /// to root node of graph, it is required because graph can contain only one root.
    #[inline]
    pub fn add_node(&mut self, node: Node) -> Handle<Node> {
        let handle = self.pool.spawn(node);
        self.link_nodes(handle, self.root);
        handle
    }

    /// Tries to borrow shared reference to a node by specified handle. Will return None if handle
    /// is invalid. Handle can be invalid for either because its index out-of-bounds or generation
    /// of handle does not match generation of node.
    pub fn get(&self, node: Handle<Node>) -> Option<&Node> {
        self.pool.borrow(node)
    }

    /// Tries to borrow mutable reference to a node by specified handle. Will return None if handle
    /// is invalid. Handle can be invalid for either because its index out-of-bounds or generation
    /// of handle does not match generation of node.
    pub fn get_mut(&mut self, node: Handle<Node>) -> Option<&mut Node> {
        self.pool.borrow_mut(node)
    }

    /// Tries to borrow mutable references to two nodes at the same time by given handles. Will
    /// return Err of handles overlaps (points to same node).
    pub fn get_two_mut(&mut self, nodes: (Handle<Node>, Handle<Node>))
                       -> Result<(Option<&mut Node>, Option<&mut Node>), ()> {
        self.pool.borrow_two_mut(nodes)
    }

    /// Tries to borrow mutable references to three nodes at the same time by given handles. Will
    /// return Err of handles overlaps (points to same node).
    pub fn get_tree_mut(&mut self, nodes: (Handle<Node>, Handle<Node>, Handle<Node>))
                        -> Result<(Option<&mut Node>, Option<&mut Node>, Option<&mut Node>), ()> {
        self.pool.borrow_three_mut(nodes)
    }

    /// Tries to borrow mutable references to four nodes at the same time by given handles. Will
    /// return Err of handles overlaps (points to same node).
    pub fn get_four_mut(&mut self, nodes: (Handle<Node>, Handle<Node>, Handle<Node>, Handle<Node>))
                        -> Result<(Option<&mut Node>, Option<&mut Node>, Option<&mut Node>, Option<&mut Node>), ()> {
        self.pool.borrow_four_mut(nodes)
    }

    /// Returns root node of current graph.
    pub fn get_root(&self) -> Handle<Node> {
        self.root
    }

    /// Destroys node and its children recursively.
    #[inline]
    pub fn remove_node(&mut self, node_handle: Handle<Node>) {
        self.stack.clear();
        self.stack.push(node_handle);
        while let Some(handle) = self.stack.pop() {
            if let Some(node) = self.pool.borrow(handle) {
                for child in node.children.iter() {
                    self.stack.push(*child);
                }
            }
            self.pool.free(handle);
        }
    }

    /// Links specified child with specified parent.
    #[inline]
    pub fn link_nodes(&mut self, child_handle: Handle<Node>, parent_handle: Handle<Node>) {
        self.unlink_nodes(child_handle);
        if let Some(child) = self.pool.borrow_mut(child_handle) {
            child.parent = parent_handle;
            if let Some(parent) = self.pool.borrow_mut(parent_handle) {
                parent.children.push(child_handle);
            }
        }
    }

    /// Unlinks specified node from its parent, so node will become root.
    #[inline]
    pub fn unlink_nodes(&mut self, node_handle: Handle<Node>) {
        let mut parent_handle = Handle::NONE;
        // Replace parent handle of child
        if let Some(node) = self.pool.borrow_mut(node_handle) {
            parent_handle = node.parent;
            node.parent = Handle::NONE;
        }
        // Remove child from parent's children list
        if let Some(parent) = self.pool.borrow_mut(parent_handle) {
            if let Some(i) = parent.children.iter().position(|h| *h == node_handle) {
                parent.children.remove(i);
            }
        }
    }

    /// Tries to find a copy of `node_handle` in hierarchy tree starting from `root_handle`.
    pub fn find_copy_of(&self, root_handle: Handle<Node>, node_handle: Handle<Node>) -> Handle<Node> {
        if let Some(root) = self.pool.borrow(root_handle) {
            if root.get_original_handle() == node_handle {
                return root_handle;
            }

            for child_handle in root.children.iter() {
                let out = self.find_copy_of(*child_handle, node_handle);
                if out.is_some() {
                    return out;
                }
            }
        }
        Handle::NONE
    }

    /// Searches node with specified name starting from specified node. If nothing was found,
    /// [`Handle::NONE`] is returned.
    pub fn find_by_name(&self, root_node: Handle<Node>, name: &str) -> Handle<Node> {
        match self.pool.borrow(root_node) {
            Some(node) => {
                if node.get_name() == name {
                    root_node
                } else {
                    let mut result: Handle<Node> = Handle::NONE;
                    for child in &node.children {
                        let child_handle = self.find_by_name(*child, name);
                        if !child_handle.is_none() {
                            result = child_handle;
                            break;
                        }
                    }
                    result
                }
            }
            None => Handle::NONE
        }
    }

    /// Searches node with specified name starting from root. If nothing was found, [`Handle::NONE`]
    /// is returned.
    pub fn find_by_name_from_root(&self, name: &str) -> Handle<Node> {
        self.find_by_name(self.root, name)
    }

    /// Creates a full copy of node with all children. This is relatively heavy operation!
    /// In case if any error happened it returns [`Handle::NONE`]. Automatically
    /// remaps bones for copied surfaces.
    pub fn copy_node(&self, node_handle: Handle<Node>, dest_graph: &mut Graph) -> Handle<Node> {
        let mut old_new_mapping: HashMap<Handle<Node>, Handle<Node>> = HashMap::new();
        let root_handle = self.copy_node_raw(node_handle, dest_graph, &mut old_new_mapping);

        // Iterate over instantiated nodes and remap bones handles.
        for (_, new_node_handle) in old_new_mapping.iter() {
            if let Some(node) = dest_graph.pool.borrow_mut(*new_node_handle) {
                if let NodeKind::Mesh(mesh) = node.get_kind_mut() {
                    for surface in mesh.get_surfaces_mut() {
                        for bone_handle in surface.bones.iter_mut() {
                            if let Some(entry) = old_new_mapping.get(bone_handle) {
                                *bone_handle = *entry;
                            }
                        }
                    }
                }
            }
        }

        root_handle
    }

    fn copy_node_raw(&self, root_handle: Handle<Node>, dest_graph: &mut Graph, old_new_mapping: &mut HashMap<Handle<Node>, Handle<Node>>) -> Handle<Node> {
        match self.pool.borrow(root_handle) {
            Some(src_node) => {
                let dest_node = src_node.make_copy(root_handle);
                let dest_copy_handle = dest_graph.add_node(dest_node);
                old_new_mapping.insert(root_handle, dest_copy_handle);
                for src_child_handle in &src_node.children {
                    let dest_child_handle = self.copy_node_raw(*src_child_handle, dest_graph, old_new_mapping);
                    if !dest_child_handle.is_none() {
                        dest_graph.link_nodes(dest_child_handle, dest_copy_handle);
                    }
                }
                dest_copy_handle
            }
            None => Handle::NONE
        }
    }

    fn find_model_root(&self, from: Handle<Node>) -> Handle<Node> {
        let mut model_root_handle = from;
        while let Some(model_node) = self.pool.borrow(model_root_handle) {
            if self.pool.borrow(model_node.get_parent()).is_none() {
                // We have no parent on node, then it must be root.
                return model_root_handle;
            }

            if model_node.is_resource_instance {
                return model_root_handle;
            }

            // Continue searching up on hierarchy.
            model_root_handle = model_node.get_parent();
        }
        model_root_handle
    }

    pub(in crate) fn resolve(&mut self) {
        println!("Resolving graph...");
        self.update_transforms();

        // Resolve original handles. Original handle is a handle to a node in resource from which
        // a node was instantiated from. We can resolve it only by names of nodes, but this is not
        // reliable way of doing this, because some editors allow nodes to have same names for
        // objects, but here we'll assume that modellers will not create models with duplicated
        // names.
        for node in self.pool.iter_mut() {
            if let Some(model) = node.get_resource() {
                let model = model.lock().unwrap();
                let SceneInterface { graph: resource_graph, .. } = model.get_scene().interface();
                for (handle, resource_node) in resource_graph.pair_iter() {
                    if resource_node.get_name() == node.get_name() {
                        node.set_original_handle(handle);
                        node.set_inv_bind_pose_transform(*resource_node.get_inv_bind_pose_transform());
                        break;
                    }
                }
            }
        }

        println!("Original handles resolved!");

        // Then iterate over all scenes and resolve changes in surface data, remap bones, etc.
        // This step is needed to take correct graphical data from resource, we do not store
        // meshes in save files, just references to resource this data was taken from. So on
        // resolve stage we just copying surface from resource, do bones remapping. Bones remapping
        // is required stage because we copied surface from resource and bones are mapped to nodes
        // in resource, but we must have them mapped to instantiated nodes on scene. To do that
        // we'll try to find a root for each node, and starting from it we'll find corresponding
        // bone nodes. I know that this sounds too confusing but try to understand it.
        for i in 0..self.pool.get_capacity() {
            let node_handle = self.pool.handle_from_index(i);

            // TODO HACK: Fool borrow checker for now.
            let mgraph = unsafe { &mut *(self as *mut Graph) };

            let root_handle = self.find_model_root(node_handle);

            if let Some(node) = self.pool.at_mut(i) {
                let node_name = String::from(node.get_name());
                if let Some(model) = node.get_resource() {
                    if let NodeKind::Mesh(mesh) = node.get_kind_mut() {
                        let model = model.lock().unwrap();
                        let resource_node_handle = model.find_node_by_name(node_name.as_str());
                        let SceneInterface { graph: resource_graph, .. } = model.get_scene().interface();
                        if let Some(resource_node) = resource_graph.get(resource_node_handle) {
                            if let NodeKind::Mesh(resource_mesh) = resource_node.get_kind() {
                                // Copy surfaces from resource and assign to meshes.
                                let surfaces = mesh.get_surfaces_mut();
                                surfaces.clear();
                                for resource_surface in resource_mesh.get_surfaces() {
                                    surfaces.push(resource_surface.make_copy());
                                }

                                // Remap bones
                                for surface in mesh.get_surfaces_mut() {
                                    for bone_handle in surface.bones.iter_mut() {
                                        *bone_handle = mgraph.find_copy_of(root_handle, *bone_handle);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        println!("Graph resolved successfully!");
    }

    pub fn update_transforms(&mut self) {
        // Calculate transforms on nodes
        self.stack.clear();
        self.stack.push(self.root);
        while let Some(handle) = self.stack.pop() {
            // Calculate local transform and get parent handle
            let mut parent_handle: Handle<Node> = Handle::NONE;
            if let Some(node) = self.pool.borrow_mut(handle) {
                parent_handle = node.parent;
            }

            // Extract parent's global transform
            let mut parent_global_transform = Mat4::identity();
            let mut parent_visibility = true;
            if let Some(parent) = self.pool.borrow(parent_handle) {
                parent_global_transform = parent.global_transform;
                parent_visibility = parent.global_visibility;
            }

            if let Some(node) = self.pool.borrow_mut(handle) {
                node.global_transform = parent_global_transform * node.local_transform.get_matrix();
                node.global_visibility = parent_visibility && node.visibility;

                // Queue children and continue traversal on them
                for child_handle in node.children.iter() {
                    self.stack.push(child_handle.clone());
                }
            }
        }
    }

    pub fn update_nodes(&mut self, aspect_ratio: f32, dt: f32) {
        self.update_transforms();

        for node in self.pool.iter_mut() {
            let eye = node.get_global_position();
            let look = node.get_look_vector();
            let up = node.get_up_vector();

            match node.get_kind_mut() {
                NodeKind::Camera(camera) => camera.calculate_matrices(eye, look, up, aspect_ratio),
                NodeKind::ParticleSystem(particle_system) => particle_system.update(dt),
                _ => ()
            }
        }
    }

    /// Creates an iterator that has linear iteration order over internal collection
    /// of nodes. It does *not* perform any tree traversal!
    pub fn linear_iter(&self) -> PoolIterator<Node> {
        self.pool.iter()
    }

    /// Creates an iterator that has linear iteration order over internal collection
    /// of nodes. It does *not* perform any tree traversal!
    pub fn linear_iter_mut(&mut self) -> PoolIteratorMut<Node> {
        self.pool.iter_mut()
    }

    /// Creates new iterator that iterates over internal collection giving (handle; node) pairs.
    pub fn pair_iter(&self) -> PoolPairIterator<Node> {
        self.pool.pair_iter()
    }
}

impl Visit for Graph {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        visitor.enter_region(name)?;

        // Pool must be empty, otherwise handles will be invalid and everything will blow up.
        if visitor.is_reading() && self.pool.get_capacity() != 0 {
            panic!("Graph pool must be empty on load!")
        }

        self.root.visit("Root", visitor)?;
        self.pool.visit("Pool", visitor)?;

        visitor.leave_region()
    }
}

#[cfg(test)]
mod test {
    use crate::scene::graph::Graph;
    use rg3d_core::pool::Handle;
    use crate::scene::node::{Node, NodeKind};

    #[test]
    fn graph_init_test() {
        let graph = Graph::new();
        assert_ne!(graph.root, Handle::NONE);
        assert_eq!(graph.pool.alive_count(), 1);
    }

    #[test]
    fn graph_node_test() {
        let mut graph = Graph::new();
        let a = graph.add_node(Node::new(NodeKind::Base));
        let b = graph.add_node(Node::new(NodeKind::Base));
        let c = graph.add_node(Node::new(NodeKind::Base));
        assert_eq!(graph.pool.alive_count(), 4);
    }
}